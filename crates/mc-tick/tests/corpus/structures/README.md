# Corpus structures

Java structure SNBT, the same format `tools/gametest` feeds to real Minecraft.

Keeping one copy that both consume is deliberate: if the engine and the oracle
read different files, a trace diff tells you nothing about behaviour.

Each of these has been run through vanilla via `tools/gametest/run.sh` or
`TraceCapture`, so the captured traces and these files describe the same build.
