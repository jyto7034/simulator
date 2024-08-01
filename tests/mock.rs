pub mod mock {
    // 게임을 테스트하기 위해선, app 객체와 server 가 필요함.
    // 근데 server 을 직접 만들기엔 시간 걸리니, 사전에 작성해둔 게임의 진행 정보를 순서데로 반환하는 객체를 하나 만들어서
    // 테스트 하기로함.
    // App 객체가 config 를 받고, 생성되는 함수를 이 mock mod 에서 활용하여 mock app 을 생성하여 리턴함.
    // config 파일은 tests 에서 작성하여 App 생성 함수로 넘김.

    // mock server
    pub struct Server{

    }

    use card_game::app::app::App;

    // mock app
    pub struct App {
        app: App,
    }
    impl App {
        pub fn instantiate() {

        }
    }
}
