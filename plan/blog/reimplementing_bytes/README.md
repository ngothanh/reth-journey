# Reimplementing `Bytes` — a zero-copy byte handle, from scratch

An investigation, in five parts, into a small but famously tricky piece of code: the
zero-copy byte handle that lives inside Rust's `bytes` crate, Facebook's `IOBuf`, and
Netty's `ByteBuf`. We start from an everyday need — a network program reading data in
and passing it around — hit a performance problem, try the obvious fixes, watch them
fail, and rebuild the real thing one wall at a time.

No prior knowledge of `Bytes`, `BytesMut`, or `freeze` needed. Part 1 starts from
zero.

## Languages / Ngôn ngữ / Sprachen / 语言

- 🇬🇧 **English** — [`en/00_index.md`](en/00_index.md)
- 🇩🇪 **Deutsch** — [`de/00_index.md`](de/00_index.md)

All versions cover the same five parts and the same ideas; each is self-contained.
Pick one and read it top to bottom.

## Parts

| # | English | Tiếng Việt | Deutsch | 简体中文 |
|---|---|---|---|---|
| 1 | [The problem](en/01_the_problem.md) | [Bài toán](vi/01_the_problem.md) | [Das Problem](de/01_the_problem.md) | [问题](zh/01_the_problem.md) |
| 2 | [One type, many behaviours](en/02_vtable.md) | [Một kiểu, nhiều hành vi](vi/02_vtable.md) | [Ein Typ, viele Verhalten](de/02_vtable.md) | [一个类型，多种行为](zh/02_vtable.md) |
| 3 | [Which bytes vs who owns](en/03_split_and_counting.md) | [Byte nào vs ai sở hữu](vi/03_split_and_counting.md) | [Welche Bytes vs. wer besitzt](de/03_split_and_counting.md) | [哪些字节 vs 谁拥有](zh/03_split_and_counting.md) |
| 4 | [The clone wall](en/04_promotion.md) | [Bức tường clone](vi/04_promotion.md) | [Die clone-Wand](de/04_promotion.md) | [clone 之墙](zh/04_promotion.md) |
| 5 | [`AtomicPtr`](en/05_atomics.md) | [`AtomicPtr`](vi/05_atomics.md) | [`AtomicPtr`](de/05_atomics.md) | [`AtomicPtr`](zh/05_atomics.md) |
