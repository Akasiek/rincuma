const port = process.env.NITRO_PORT || "3000";

try {
  const response = await fetch(`http://127.0.0.1:${port}/api/health`, {
    signal: AbortSignal.timeout(5000),
    redirect: "error",
  });
  process.exitCode = response.status === 200 ? 0 : 1;
} catch {
  process.exitCode = 1;
}
