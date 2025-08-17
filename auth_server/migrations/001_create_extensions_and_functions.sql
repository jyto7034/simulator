-- 데이터베이스에 uuid-ossp 확장 기능 활성화 (여전히 다른 테이블에서 UUID를 사용할 수 있으므로 유지)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- updated_at 컬럼 자동 갱신을 위한 트리거 함수
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';