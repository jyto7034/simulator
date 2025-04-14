use actix::clock::{sleep, Instant};
use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// --- 메시지 정의 ---

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
struct AuctionStatus {
    item: String,
    highest_bid: u64,
    highest_bidder: Option<Uuid>,
    time_remaining_secs: u64,
    is_open: bool,
    winner: Option<Uuid>, // 경매 종료 시 사용
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
struct StartAuction {
    item: String,
    starting_bid: u64,
    duration_secs: u64,
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")] // 성공 또는 에러 메시지 반환
struct PlaceBid {
    bidder_id: Uuid,
    amount: u64,
}

#[derive(Message)]
#[rtype(result = "AuctionStatus")]
struct GetStatus;

#[derive(Message)]
#[rtype(result = "()")]
struct AuctionEnded {
    item: String,
    winner: Option<Uuid>,
    winning_bid: u64,
}

// 내부 타이머 메시지
#[derive(Message)]
#[rtype(result = "()")]
struct InternalTimerTick;

// --- Auctioneer 액터 정의 ---

struct AuctioneerActor {
    item: Option<String>,
    starting_bid: u64,
    highest_bid: u64,
    highest_bidder: Option<Uuid>,
    bidders: HashMap<Uuid, Recipient<AuctionStatus>>, // 참여 입찰자 (상태 업데이트 수신용)
    is_open: bool,
    end_time: Option<Instant>,         // Instant 사용으로 변경
    timer_handle: Option<SpawnHandle>, // 타이머 핸들 관리
}

impl AuctioneerActor {
    fn new() -> Self {
        AuctioneerActor {
            item: None,
            starting_bid: 0,
            highest_bid: 0,
            highest_bidder: None,
            bidders: HashMap::new(),
            is_open: false,
            end_time: None,
            timer_handle: None,
        }
    }

    // 현재 경매 상태를 담은 메시지 생성
    fn get_current_status(&self) -> AuctionStatus {
        let time_remaining = if let Some(et) = self.end_time {
            // Instant::now() 대신 Context::clock() 사용 가능 (테스트 용이)
            et.saturating_duration_since(Instant::now()).as_secs()
        } else {
            0
        };

        AuctionStatus {
            item: self.item.clone().unwrap_or_default(),
            highest_bid: self.highest_bid,
            highest_bidder: self.highest_bidder,
            time_remaining_secs: time_remaining,
            is_open: self.is_open,
            winner: if !self.is_open {
                self.highest_bidder
            } else {
                None
            },
        }
    }

    // 모든 참여자에게 상태 브로드캐스트
    fn broadcast_status(&self, _ctx: &mut Context<Self>) {
        if !self.is_open && self.timer_handle.is_some() {
            println!("AUCTIONEER: Attempted to broadcast status but auction is closed or ending.");
            return; // 경매가 막 종료되었거나 시작되지 않았으면 브로드캐스트 안 함
        }

        let status = self.get_current_status();
        println!("AUCTIONEER: Broadcasting status: {:?}", status);
        for recipient in self.bidders.values() {
            println!("AUCTIONEER: Sending status to recipient.");
            recipient.do_send(status.clone());
        }
    }

    // 경매 종료 처리
    fn end_auction(&mut self, ctx: &mut Context<Self>) {
        if !self.is_open {
            return;
        } // 이미 종료됨

        println!(
            "AUCTIONEER: Auction for '{}' ended.",
            self.item.as_ref().unwrap_or(&"N/A".to_string())
        );
        self.is_open = false;

        // 타이머 중지
        if let Some(handle) = self.timer_handle.take() {
            ctx.cancel_future(handle);
            println!("AUCTIONEER: Timer cancelled.");
        }

        let final_status = self.get_current_status(); // winner 정보 포함
        println!("AUCTIONEER: Final Status: {:?}", final_status);

        // 최종 결과 브로드캐스트
        for recipient in self.bidders.values() {
            recipient.do_send(final_status.clone());
        }

        // 경매 상태 초기화 (다음 경매 준비) - 선택적
        // self.item = None;
        // self.highest_bid = 0;
        // self.highest_bidder = None;
        // self.end_time = None;
        // self.bidders.clear(); // 참여자 목록 초기화 여부는 정책에 따라 다름
    }
}

impl Actor for AuctioneerActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("AuctioneerActor started.");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("AuctioneerActor stopped.");
    }
}

// --- 메시지 핸들러 구현 ---

impl Handler<StartAuction> for AuctioneerActor {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: StartAuction, ctx: &mut Context<Self>) -> Self::Result {
        if self.is_open {
            return Err("Another auction is already in progress.".to_string());
        }

        println!(
            "AUCTIONEER: Starting auction for '{}', starting bid: {}, duration: {}s",
            msg.item, msg.starting_bid, msg.duration_secs
        );

        self.item = Some(msg.item);
        self.starting_bid = msg.starting_bid;
        self.highest_bid = msg.starting_bid; // 시작가를 초기 최고가로 설정
        self.highest_bidder = None;
        self.is_open = true;
        self.end_time = Some(Instant::now() + Duration::from_secs(msg.duration_secs));
        self.bidders.clear(); // 새 경매 시작 시 참여자 초기화

        // 기존 타이머 취소 (혹시 모르니)
        if let Some(handle) = self.timer_handle.take() {
            ctx.cancel_future(handle);
        }

        // 1초마다 InternalTimerTick 메시지를 자신에게 보내는 타이머 설정
        self.timer_handle = Some(ctx.run_interval(Duration::from_secs(1), |act, ctx| {
            // 타이머가 만료되었는지 확인
            if let Some(et) = act.end_time {
                if Instant::now() >= et {
                    println!("AUCTIONEER: Timer expired.");
                    act.end_auction(ctx); // 타이머 만료 시 경매 종료
                } else {
                    // 아직 진행 중이면 상태 브로드캐스트 (선택적: 너무 자주 보낼 수 있음)
                    // act.broadcast_status(ctx);
                    // 대신 InternalTimerTick 메시지를 보내서 처리하게 할 수도 있음
                    ctx.notify(InternalTimerTick);
                }
            } else {
                // 종료 시간이 없으면 타이머 중지 (이론상 발생하면 안 됨)
                println!("AUCTIONEER: ERROR - Timer running without end_time!");
                if let Some(handle) = act.timer_handle.take() {
                    ctx.cancel_future(handle);
                }
            }
        }));
        println!("AUCTIONEER: Timer started.");

        // 즉시 상태 브로드캐스트
        self.broadcast_status(ctx);

        Ok(())
    }
}

// 입찰자 등록 및 입찰 처리
impl Handler<PlaceBid> for AuctioneerActor {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: PlaceBid, ctx: &mut Context<Self>) -> Self::Result {
        if !self.is_open {
            return Err("Auction is not open.".to_string());
        }
        if Instant::now() >= self.end_time.unwrap() {
            return Err("Auction has already ended.".to_string());
        }

        // 입찰자가 처음 입찰하는 경우 등록 (Recipient 필요)
        // 실제로는 BidderActor의 Addr를 받아서 Recipient를 만들어야 함.
        // if !self.bidders.contains_key(&msg.bidder_id) {
        //     // self.bidders.insert(msg.bidder_id, bidder_recipient);
        //     println!("AUCTIONEER: Bidder {} registered.", msg.bidder_id);
        // }

        // 입찰가 유효성 검사
        if msg.amount <= self.highest_bid {
            return Err(format!(
                "Your bid ({}) must be higher than the current highest bid ({}).",
                msg.amount, self.highest_bid
            ));
        }

        println!(
            "AUCTIONEER: Bid received from {}: {}",
            msg.bidder_id, msg.amount
        );
        self.highest_bid = msg.amount;
        self.highest_bidder = Some(msg.bidder_id);

        // 새로운 최고가 발생 시 상태 브로드캐스트
        self.broadcast_status(ctx);

        Ok(())
    }
}

impl Handler<GetStatus> for AuctioneerActor {
    type Result = MessageResult<GetStatus>;

    fn handle(&mut self, _msg: GetStatus, _ctx: &mut Context<Self>) -> Self::Result {
        println!("AUCTIONEER: GetStatus request received.");
        MessageResult(self.get_current_status())
    }
}

// 내부 타이머 틱 처리 (선택적: 상태 업데이트용)
impl Handler<InternalTimerTick> for AuctioneerActor {
    type Result = ();
    fn handle(&mut self, _msg: InternalTimerTick, ctx: &mut Context<Self>) -> Self::Result {
        // 타이머 틱마다 상태를 브로드캐스트 할 수 있음
        // run_interval 클로저 대신 여기서 처리하면 로직 분리 가능
        if self.is_open {
            self.broadcast_status(ctx);
        }
    }
}

// --- Bidder 액터 정의 (간단화) ---
struct BidderActor {
    id: Uuid,
    name: String,
    auctioneer_addr: Addr<AuctioneerActor>,
    last_status: Option<AuctionStatus>,
}

impl Actor for BidderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("BIDDER [{} / {}]: Started.", self.name, self.id);
        // 시작 시 경매 상태 요청
        self.request_status(ctx);
    }
}

// AuctionStatus 메시지 핸들러 (Auctioneer로부터 받음)
impl Handler<AuctionStatus> for BidderActor {
    type Result = ();
    fn handle(&mut self, msg: AuctionStatus, _ctx: &mut Context<Self>) -> Self::Result {
        println!("BIDDER [{} / {}]: Received Status - Item: '{}', Highest Bid: {}, Bidder: {:?}, Time Left: {}s, Open: {}, Winner: {:?}",
                 self.name, self.id, msg.item, msg.highest_bid, msg.highest_bidder, msg.time_remaining_secs, msg.is_open, msg.winner);
        self.last_status = Some(msg);

        // TODO: 여기에 입찰 결정 로직 추가 가능
        // 예: if status.is_open && status.highest_bidder != Some(self.id) { self.place_bid(...) }
    }
}

// 입찰 로직 (헬퍼 함수)
impl BidderActor {
    fn place_bid(&self, ctx: &mut Context<Self>, amount: u64) {
        println!(
            "BIDDER [{} / {}]: Attempting to bid {}.",
            self.name, self.id, amount
        );
        // Auctioneer에게 PlaceBid 메시지 보내기
        let bid_msg = PlaceBid {
            bidder_id: self.id,
            amount,
        };
        self.auctioneer_addr
            .send(bid_msg)
            .into_actor(self) // 결과를 이 액터의 컨텍스트에서 처리하도록 함
            .then(|res, _act, _ctx| {
                // 결과 처리 클로저
                match res {
                    Ok(Ok(())) => println!("BIDDER [{}]: Bid successful!", _act.name),
                    Ok(Err(e)) => println!("BIDDER [{}]: Bid failed - {}", _act.name, e),
                    Err(e) => println!("BIDDER [{}]: MailboxError sending bid: {}", _act.name, e),
                }
                fut::ready(()) // Future 완료
            })
            .wait(ctx); // Future가 완료될 때까지 기다리지 않고, 완료되면 실행되도록 등록
                        // wait()는 현재 메시지 처리를 중단시키지 않음
    }

    fn request_status(&self, ctx: &mut Context<Self>) {
        println!(
            "BIDDER [{} / {}]: Requesting auction status.",
            self.name, self.id
        );
        self.auctioneer_addr
            .send(GetStatus)
            .into_actor(self)
            .then(|res, act, _ctx| {
                match res {
                    Ok(status) => {
                        println!(
                            "BIDDER [{}]: Received status on request: {:?}",
                            act.name, status
                        );
                        act.last_status = Some(status);
                    }
                    Err(e) => println!("BIDDER [{}]: Failed to get status: {}", act.name, e),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

// --- 메인 함수 (테스트용) ---
#[actix_web::main]
async fn main() {
    println!("--- Starting Auction System ---");

    // Auctioneer 시작
    let auctioneer_addr = AuctioneerActor::new().start();

    // Bidder 시작
    let bidder1_id = Uuid::new_v4();
    let bidder1_addr = BidderActor {
        id: bidder1_id,
        name: "Alice".to_string(),
        auctioneer_addr: auctioneer_addr.clone(),
        last_status: None,
    }
    .start();

    let bidder2_id = Uuid::new_v4();
    let bidder2_addr = BidderActor {
        id: bidder2_id,
        name: "Bob".to_string(),
        auctioneer_addr: auctioneer_addr.clone(),
        last_status: None,
    }
    .start();

    // Auctioneer에게 Bidder Recipient 등록 (원래는 Bidder가 Join 같은 메시지를 보내야 함)
    println!("\n--- Manually Registering Bidders (for simplicity) ---");
    // 실제로는 Auctioneer가 직접 상태를 수정하는 대신 메시지를 사용해야 함.
    // auctioneer_addr.do_send(RegisterBidder { id: bidder1_id, recipient: bidder1_addr.recipient() });
    // auctioneer_addr.do_send(RegisterBidder { id: bidder2_id, recipient: bidder2_addr.recipient() });
    // 위와 같이 직접 등록하는 대신, StartAuction 핸들러에서 bidders 맵을 초기화하고,
    // PlaceBid 핸들러에서 처음 입찰하는 bidder를 등록하도록 수정했습니다.
    // (단, Recipient를 얻으려면 BidderActor의 Addr가 필요하므로, PlaceBid에 Addr를 포함시키거나
    //  별도의 Register 메시지를 구현해야 합니다. 이 예제에서는 단순화를 위해 Recipient 등록 생략)

    // 경매 시작
    println!("\n--- Starting Auction ---");
    let start_res = auctioneer_addr
        .send(StartAuction {
            item: "Rare Pepe".to_string(),
            starting_bid: 100,
            duration_secs: 5, // 짧은 시간 설정
        })
        .await;
    println!("StartAuction Result: {:?}", start_res);

    // Bidder에게 Recipient 전달 (실제로는 Join 응답 등으로 받아야 함)
    // 이 예제에서는 BidderActor 생성 시 auctioneer_addr를 전달받음.
    // 상태 업데이트를 받으려면 Auctioneer가 Bidder의 Recipient를 알아야 함.
    // 이 부분은 예제의 한계입니다. 실제로는 등록 과정이 필요합니다.

    // 입찰 시뮬레이션
    println!("\n--- Bidding Simulation ---");
    // Alice 입찰 (BidderActor 내부에서 place_bid 호출하도록 수정 필요)
    bidder1_addr.do_send(PlaceBidInternal(110)); // Actor 내부에서 호출하도록 메시지 추가
    sleep(Duration::from_millis(500)).await;

    // Bob 입찰
    bidder2_addr.do_send(PlaceBidInternal(120));
    sleep(Duration::from_millis(500)).await;

    // Alice 다시 입찰
    bidder1_addr.do_send(PlaceBidInternal(150));
    sleep(Duration::from_millis(500)).await;

    println!("\n--- Waiting for auction to end ({} seconds) ---", 5);
    sleep(Duration::from_secs(6)).await; // 경매 시간 + 여유 시간

    println!("\n--- Requesting final status ---");
    bidder1_addr.do_send(RequestStatusInternal);
    bidder2_addr.do_send(RequestStatusInternal);

    sleep(Duration::from_millis(100)).await; // 상태 요청 처리 시간

    println!("\n--- Stopping System ---");
    System::current().stop();
}

// --- Bidder 내부 로직 호출을 위한 메시지 ---
#[derive(Message)]
#[rtype(result = "()")]
struct PlaceBidInternal(u64);

impl Handler<PlaceBidInternal> for BidderActor {
    type Result = ();
    fn handle(&mut self, msg: PlaceBidInternal, ctx: &mut Context<Self>) -> Self::Result {
        self.place_bid(ctx, msg.0);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct RequestStatusInternal;

impl Handler<RequestStatusInternal> for BidderActor {
    type Result = ();
    fn handle(&mut self, _msg: RequestStatusInternal, ctx: &mut Context<Self>) -> Self::Result {
        self.request_status(ctx);
    }
}
