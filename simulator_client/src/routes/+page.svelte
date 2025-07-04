<script lang="ts">
	import { onMount } from 'svelte';
	import { fetch } from '@tauri-apps/plugin-http';
	import { invoke } from '@tauri-apps/api/core';

	let loginStatus: 'idle' | 'pending' | 'success' | 'error' = 'idle';
	let statusMessage = '';
	let steamId = '';

	async function handleLogin() {
		loginStatus = 'pending';
		statusMessage = '로그인 중...';

		try {
			// Tauri 커맨드를 호출하여 Steam 티켓을 가져옵니다.
			const [id, ticket] = await invoke<(number | string)[]>('get_steam_ticket');
			steamId = String(id); // Steam ID를 상태에 저장

			const authServerUrl = 'http://localhost:3000/auth';

			const response = await fetch(`${authServerUrl}/steam`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ ticket })
			});

			if (response.ok) {
				const data = (await response.json()) as { message: string; steam_id: string };
				loginStatus = 'success';
				statusMessage = `로그인 성공! (서버 메시지: ${data.message})`;
				steamId = data.steam_id;
			} else {
				loginStatus = 'error';
				const errorText = await response.json(); // 오류 응답도 json일 수 있으므로 파싱
				statusMessage = `로그인 실패: ${response.status} - ${JSON.stringify(errorText)}`;
			}
		} catch (e) {
			loginStatus = 'error';
			statusMessage = `로그인 요청 중 오류 발생: ${e}`;
		}
	}

	onMount(() => {
		handleLogin();
	});
</script>

<div class="container mx-auto p-4 flex flex-col items-center justify-center min-h-screen">
	<div class="card w-96 bg-base-100 shadow-xl">
		<div class="card-body">
			<h2 class="card-title">Tauri App</h2>

			{#if loginStatus === 'pending'}
				<div class="flex items-center space-x-2">
					<span class="loading loading-spinner loading-md"></span>
					<p>{statusMessage}</p>
				</div>
			{:else if loginStatus === 'success'}
				<div role="alert" class="alert alert-success">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-6 w-6 shrink-0 stroke-current"
						fill="none"
						viewBox="0 0 24 24"
						><path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
						/></svg
					>
					<span>{statusMessage}</span>
				</div>
				<p class="text-lg">Steam ID: <span class="font-bold">{steamId}</span></p>
			{:else if loginStatus === 'error'}
				<div role="alert" class="alert alert-error">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-6 w-6 shrink-0 stroke-current"
						fill="none"
						viewBox="0 0 24 24"
						><path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
						/></svg
					>
					<span>{statusMessage}</span>
				</div>
			{/if}

			<div class="card-actions justify-end mt-4">
				<button class="btn btn-primary" on:click={handleLogin} disabled={loginStatus === 'pending'}>
					{#if loginStatus === 'pending'}
						<span class="loading loading-spinner"></span>
					{/if}
					다시 시도
				</button>
			</div>
		</div>
	</div>
</div>
