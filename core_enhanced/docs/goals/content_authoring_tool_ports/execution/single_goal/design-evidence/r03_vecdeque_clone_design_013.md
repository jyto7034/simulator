# R03 VecDeque native Clone 설계013

AUTORUN §8의 제한 읽기 조사이며 완료 감사가 아니다. 설치된 Rust rustdoc/source HTML을 읽었으며 source/docs/Git 변경, cargo/build/test 실행, network/download는 하지 않았다.

**최소 visitor는 기존 Vec visitor와 같은 checked(len + 1) container storage 계수 후 borrowed iter()로 자식 방문이다.** Native VecDeque::clone은 source capacity와 wrap 위치를 재현하지 않고, live len만큼 새 버퍼를 만들며 front-to-back 순서로 element를 clone한다. RecruitmentState.queue에 lifetime allocation history는 필요 없다. 이 판단은 native clone에 관한 것이며 clone_from이나 source+destination 동시 보유 총량의 증명은 아니다.

## 설치 원문과 실제 호출 경로

로컬 rustdoc root:
`/home/acr0tic/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/`

`std/collections/struct.VecDeque.html`의 metadata는 **1.91.1 (ed61e7d7e 2025-11-07)**이다. 실행 중인 rustc binary를 별도로 호출한 것은 아니다. 아래 span은 `src/...rs.html`에 표시된 Rust source line이며 HTML을 파싱할 때 line-number anchor만 제외했다. 원본 HTML와 추출 source의 SHA-256은 동반 JSON에 있다.

- [VecDeque source](https://doc.rust-lang.org/1.91.1/src/alloc/collections/vec_deque/mod.rs.html):105–110: `with_capacity_in(self.len(), self.allocator().clone())` → `extend(self.iter().cloned())`. :637–638에서 새 deque의 head=0, len=0이며 RawVec::with_capacity_in에 요청 len을 넘긴다.
- [RawVec source](https://doc.rust-lang.org/1.91.1/src/alloc/raw_vec/mod.rs.html):188–192,421–430,453–488: 요청 capacity에 대한 Layout을 만들고 allocate하며, :486은 그 요청 capacity를 그대로 저장한다. size=0이면 :466–468에서 allocation하지 않는다. :512–514의 ZST capacity=usize::MAX는 논리적 특수값이며 실제 그만큼 메모리를 할당하지 않는다.
- VecDeque Extend(:3143–3146)는 [SpecExtend](https://doc.rust-lang.org/1.91.1/src/alloc/collections/vec_deque/spec_extend.rs.html):46–79에 위임한다. Iter는 TrustedLen이고 [Cloned](https://doc.rust-lang.org/1.91.1/src/core/iter/adapters/cloned.rs.html):137–143도 이를 보존한다. exact additional=len을 reserve한다. 새 destination은 이미 capacity=len, current len=0이므로 VecDeque::reserve(:834–845)는 재할당하지 않는다.
- VecDeque::iter(:1287–1289)는 as_slices(:1351–1355)의 두 slice로 [Iter](https://doc.rust-lang.org/1.91.1/src/alloc/collections/vec_deque/iter.rs.html):13–20를 만든다. Iter::next(:88–104), fold(:124–129)는 첫 slice 뒤 두 번째 slice를 읽는다. borrowed iterator는 두 slice iterator만 소유하며 heap buffer를 만들지 않는다. size_hint/ExactSizeIterator/TrustedLen은 :118–122,208–222.
- Cloned::next/fold(:40–61)가 그 순서로 T::clone을 호출한다. VecDeque::write_iter(:440–449)/write_iter_wrapping(:460–494)가 destination에 차례로 쓴다. 기존 wrap된 source는 바뀌지 않으며 clone의 head=0, source front-to-back 순서가 보존된다. source의 as_slices 분할점/capacity 보존은 native Clone의 의미가 아니다.

## Recruitment와 CloneShape 연결

`core_enhanced/src/game/recruitment.rs`:21–25의 RecruitmentState는 derived Clone이며 private queue: VecDeque<String>와 active_session을 소유한다. initialize(:47–57)의 Vec→deque 변환 뒤 pop_front(:86–89), extend(:110–121)가 wrap/여유 capacity를 만들 수 있다. 이는 위 clone bound에 영향이 없다.

현재 world/state.rs:1428–1441의 RunState derived Clone은 RecruitmentState를 값으로 복제한다. checkpoint capture macro(:388–430)의 field clone에도 recruitment(:488)가 들어간다. with_persistent_run_state_from(:1522–1526)에도 명시 recruitment.clone이 있다. 따라서 native 경로는 **RunState/checkpoint clone → RecruitmentState derived Clone → VecDeque<String>::clone → front-to-back String::clone**이다.

제안 코드는 구현 지시가 아닌 최소 형태다:

```rust
impl<T: CloneShape> CloneShape for VecDeque<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(self.len().checked_add(1))?;
        for value in self.iter() {
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}
```

여기서 +1은 기존 CloneMeter의 container-record 단위이다. String child visitor가 각각의 문자열 copy work를 더한다. 전체 visitor를 끝낸 뒤 기존 controlled_clone의 meter.finish → unchanged value.clone 순서를 유지한다(clone_shape.rs:24–34). visit 전에 clone/collect/make_contiguous를 호출할 필요가 없다.

제한:

- default allocator의 VecDeque<T> 범위다. 별도 allocator A의 Clone까지 포괄하는 blanket 구현으로 확장하려면 A의 소유·복제 비용 근거가 필요하다.
- 모든 정상 valid deque에 적용하며 fresh/deletion-free provenance는 불필요하다. empty source에 큰 spare capacity가 있어도 clone은 새 allocation 없이 empty다.
- ZST에도 len+1은 보수적인 논리 slot/work 단위다. checked overflow는 기존 meter에서 거절하며 saturate-to-zero하지 않는다.
- **clone_from은 다르다**(:118–120): destination.clear() 뒤 extend하여 기존 destination allocation을 재사용한다. destination 전체 retained storage를 source len만으로 계수할 수 없다. 이 조사에서는 clone_from admission을 설계하지 않았다.
- 실제 allocator usable bytes, RSS, 모든 미래 std 버전 또는 child CloneShape의 정확성을 이 식이 증명하지 않는다.

## Capture와 drift

Initial hash capture **2026-09-10 09:57:00.773308 UTC**, world/state 추가 capture **09:57:41.827358 UTC**, close **09:58:03.695420 UTC**.

9개 source/artifact 재해시 중 installed rustdoc 6개와 state.rs는 동일했다. Root concurrent 구현으로 recruitment.rs와 clone_shape.rs 2개가 drift했다. 보고서의 해당 source spans는 읽은 predecessor이며 successor 구현을 검토하거나 통과 판정하지 않았다. 각 before/after SHA-256을 JSON에 보존했다. Wrapped queue, empty-with-capacity, clone child order 같은 focused 실행 검증은 하지 않았다.
