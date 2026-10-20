# Avisos de terceiros

O código próprio do EMI Filter tem a licença MIT indicada em `LICENSE`.
Bibliotecas, fontes e runtimes de terceiros mantêm suas próprias licenças.
Os textos abaixo acompanham a distribuição e não alteram a licença desses
componentes. Os arquivos copiados de upstream foram preservados sem alteração.

## Bibliotecas Rust

As dependências diretas são `eframe 0.31.1`, `image 0.25.10`,
`num-complex 0.4.6`, `serde 1.0.229` e `serde_json 1.0.154`.
Essas versões declaram `MIT OR Apache-2.0`; `eframe` incorpora componentes
com condições adicionais, incluindo as fontes descritas a seguir.

[Inventário das dependências e das licenças declaradas](licenses/RUST_DEPENDENCIES.md)
registra a resolução Cargo para Linux x86_64 e Windows x86_64. Os textos de
licença e avisos encontrados nos pacotes estão em `licenses/rust/`.
Os textos comuns da família egui 0.31.1 estão em
[MIT](licenses/egui-0.31.1/LICENSE-MIT) e
[Apache-2.0](licenses/egui-0.31.1/LICENSE-APACHE), obtidos do
[upstream dessa versão](https://github.com/emilk/egui/tree/0.31.1).

O inventário inclui dependências de compilação e pacotes resolvidos pelo
Cargo. Ele não é uma lista dos objetos efetivamente retidos pelo linker,
nem uma declaração de auditoria completa de todos os componentes externos.
A biblioteca padrão Rust tem seus avisos preservados em
[COPYRIGHT-library.html](licenses/rust-standard-library/COPYRIGHT-library.html).

## Fontes incorporadas

O recurso `default_fonts` usa as fontes distribuídas no pacote
[`epaint_default_fonts 0.31.1`](https://crates.io/crates/epaint_default_fonts/0.31.1).
Sua declaração de licença é
`(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`.

| Fonte | Atribuição | Texto preservado |
|---|---|---|
| Ubuntu Light | Copyright 2011 Canonical Ltd. | [Ubuntu Font Licence 1.0](licenses/fonts/UFL.txt) |
| Noto Emoji | Copyright 2013 Google Inc. All Rights Reserved. | [SIL Open Font License 1.1](licenses/fonts/OFL.txt) |
| Hack Regular | Copyright 2018 Source Foundry Authors; contribuições DejaVu e Bitstream Vera | [Avisos MIT e Bitstream Vera completos](licenses/fonts/Hack-Regular.txt) |
| Emoji icon font | Copyright (c) 2014 John Slegers | [Licença MIT e aviso original](licenses/fonts/emoji-icon-font-mit-license.txt) |

[ATTRIBUTION.txt](licenses/fonts/ATTRIBUTION.txt) registra a proveniência dos
avisos, incluindo os copyrights extraídos dos registros originais das fontes.
Somente textos de licença e atribuição foram acrescentados a `licenses/`;
os arquivos de fonte não foram copiados para essa pasta.

## Runtime da compilação Windows

A compilação cruzada usa o
[LLVM-MinGW UCRT 20261006](https://github.com/mstorsjo/llvm-mingw/releases/tag/20261006).
O executável liga os runtimes compiler-rt, LLVM libunwind e winpthreads
estaticamente, além das partes necessárias do runtime MinGW-w64.
Os arquivos seguintes acompanham a distribuição:

- [Licença LLVM do pacote da toolchain](licenses/windows-runtime/LLVM-LICENSE.TXT).
- [Licença compiler-rt](licenses/windows-runtime/compiler-rt-LICENSE.TXT).
- [Licença LLVM libunwind](licenses/windows-runtime/libunwind-LICENSE.TXT).
- [Avisos do runtime MinGW-w64](licenses/windows-runtime/COPYING.MinGW-w64-runtime.txt).
- [Licença e avisos winpthreads](licenses/windows-runtime/COPYING.winpthreads.txt).

Os textos compiler-rt e libunwind foram obtidos do commit LLVM
`0d261d1ca552c95a8f007e061c787ac7132fbcbc`, identificado pelo compilador
23.1.3 da toolchain usada:
[compiler-rt](https://github.com/llvm/llvm-project/blob/0d261d1ca552c95a8f007e061c787ac7132fbcbc/compiler-rt/LICENSE.TXT) e
[libunwind](https://github.com/llvm/llvm-project/blob/0d261d1ca552c95a8f007e061c787ac7132fbcbc/libunwind/LICENSE.TXT).
Os demais textos vieram diretamente da distribuição LLVM-MinGW citada.
Os avisos agregados do MinGW foram mantidos inteiros, sem afirmar que cada
rotina mencionada neles esteja presente neste executável.

## Distribuição dos avisos

Os pacotes de download devem manter este arquivo, `LICENSE` e `licenses/`
junto do executável. `licenses/SHA256SUMS` permite verificar os textos
preservados a partir dessa pasta. Os avisos não acrescentam dependências
de execução ao programa.

As bibliotecas gráficas e de sistema carregadas do Linux ou do Windows são
fornecidas pelo próprio sistema; seus binários não fazem parte deste
repositório. Alterações de dependências, fontes, alvos ou toolchain exigem
revisar novamente os avisos correspondentes.
