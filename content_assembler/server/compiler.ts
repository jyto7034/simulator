import { spawn } from "node:child_process";
import type { IncomingMessage, ServerResponse } from "node:http";
import type { Plugin } from "vite";

// The executable is configured by the local developer, never by HTTP input.
export function runCompiler(
  executable: string,
  input: string,
): Promise<unknown> {
  if (Buffer.byteLength(input) > 7_000_000)
    return Promise.reject(new Error("컴파일 요청이 너무 큽니다."));
  return new Promise((resolve, reject) => {
    const child = spawn(executable, [], { stdio: ["pipe", "pipe", "pipe"] });
    const chunks: Buffer[] = [];
    let size = 0;
    let errorText = "";
    const timeout = setTimeout(() => {
      child.kill();
      reject(new Error("Core 컴파일 제한 시간을 초과했습니다."));
    }, 30_000);
    child.on("error", (error) => {
      clearTimeout(timeout);
      reject(
        new Error(`Core 컴파일러를 실행하지 못했습니다: ${error.message}`),
      );
    });
    child.stdout.on("data", (chunk: Buffer) => {
      size += chunk.length;
      if (size > 7_000_000) {
        child.kill();
        reject(new Error("Core 응답 크기 제한을 초과했습니다."));
      } else chunks.push(chunk);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      if (errorText.length < 16_000) errorText += chunk.toString("utf8");
    });
    child.on("close", (code) => {
      clearTimeout(timeout);
      if (code !== 0) {
        reject(
          new Error(errorText.trim() || "Core 컴파일러가 종료되었습니다."),
        );
        return;
      }
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
      } catch {
        reject(new Error("Core 컴파일 응답을 읽지 못했습니다."));
      }
    });
    child.stdin.on("error", (error) => reject(error));
    child.stdin.end(input);
  });
}
export function compilerPlugin(executable: string | undefined): Plugin {
  const middleware = (
    req: IncomingMessage,
    res: ServerResponse,
    next: () => void,
  ) => {
    if (req.url?.split("?")[0] !== "/__content/compile") return next();
    const host = req.headers.host ?? "";
    if (
      req.method !== "POST" ||
      req.headers["x-content-assembler"] !== "compile" ||
      !/^(localhost|127\.0\.0\.1|\[::1\])(:\d+)?$/.test(host) ||
      (req.headers.origin &&
        ![`http://${host}`, `https://${host}`].includes(req.headers.origin))
    ) {
      res.writeHead(403).end();
      return;
    }
    void (async () => {
      res.setHeader("Content-Type", "application/json; charset=utf-8");
      res.setHeader("Cache-Control", "no-store");
      try {
        if (!executable)
          throw new Error(
            "Core 컴파일러가 연결되지 않았습니다. 데스크톱 앱을 사용하거나 개발 서버의 CONTENT_ASSEMBLER_COMPILER를 설정하세요.",
          );
        const chunks: Buffer[] = [];
        let size = 0;
        for await (const chunk of req) {
          const buffer = Buffer.from(chunk);
          size += buffer.length;
          if (size > 7_000_000) throw new Error("컴파일 요청이 너무 큽니다.");
          chunks.push(buffer);
        }
        res.end(
          JSON.stringify(
            await runCompiler(
              executable,
              Buffer.concat(chunks).toString("utf8"),
            ),
          ),
        );
      } catch (error) {
        res.end(
          JSON.stringify({
            error: error instanceof Error ? error.message : String(error),
          }),
        );
      }
    })();
  };
  return {
    name: "core-graph-compiler",
    configureServer(server) {
      server.middlewares.use(middleware);
    },
    configurePreviewServer(server) {
      server.middlewares.use(middleware);
    },
  };
}
