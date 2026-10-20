# Compilação Windows

O arquivo `dist/emi-filter-windows-x86_64.exe` é um executável PE32+ de
interface gráfica, para Windows 10/11 de 64 bits. O executável contém o
programa, o cálculo e os recursos usados pela interface. Não usa Python.

O mínimo Windows 10 vem do alvo atual `x86_64-pc-windows-gnu` da biblioteca
padrão Rust, conforme a [tabela oficial de plataformas do Rust](https://doc.rust-lang.org/stable/rustc/platform-support.html).
Embora o winit 0.30 suporte versões anteriores, isso não reduz o requisito
do programa completo. O cabeçalho PE informa versão 6.00 de subsistema;
esse campo sozinho não representa o mínimo real de execução.

## Reproduzir a partir do Linux

Com Rust/Cargo, `curl`, `tar` com suporte xz e `sha256sum` disponíveis:

```sh
./build-windows.sh
```

O script instala o alvo Rust e baixa o [LLVM-MinGW oficial, release 20261006](https://github.com/mstorsjo/llvm-mingw/releases/tag/20261006),
variante UCRT para Linux x86_64. A toolchain fica no cache do usuário,
fora do projeto. O resultado usa `Cargo.lock` e é copiado para `dist/`.
Os caminhos de origem sob o diretório pessoal do usuário são remapeados
para `/build` por `--remap-path-prefix`, evitando gravar esses caminhos
locais nas mensagens incorporadas ao executável.

O SHA-256 fixado no script para o arquivo da toolchain é
`5f9c6ed95b2d4bdb2869a488c5fd5857fbdabcf288a0aa3eb1da43f6a08d8ab4`.
Esse valor foi obtido do campo `digest` publicado para o asset na
[API oficial da release](https://api.github.com/repos/mstorsjo/llvm-mingw/releases/tags/20261006)
e conferido com o SHA-256 calculado localmente após o download.

O alvo GNU do Rust pede `libgcc.a` e `libgcc_eh.a`; o script fornece os
equivalentes estáticos compiler-rt e LLVM unwind. Winpthreads também é
ligado estaticamente. A inspeção de imports do executável gerado mostrou
somente bibliotecas do Windows e da UCRT distribuída com o Windows 10/11;
não há DLLs libgcc, libstdc++, libwinpthread ou runtimes externos para
distribuir ao lado do `.exe`.

O backend gráfico é Glow/OpenGL e depende do driver gráfico do sistema.
Uma compilação bem-sucedida e a inspeção PE/imports não substituem um
teste de execução em uma máquina Windows. Nesta entrega, o executável
Windows foi compilado e inspecionado a partir do Linux; sua execução no
Windows não foi verificada.

## Artefato final verificado

Compilação de publicação da versão 1.0.0 em 20/10/2026, com as fontes
finais da interface e do modelo:

- Arquivo: `dist/emi-filter-windows-x86_64.exe`.
- Tamanho: 4.924.416 bytes.
- Formato: PE32+ x86-64, subsistema Windows GUI.
- SHA-256: `3f4b518588db97581b9bfe3c80e63aff7b200834258850ab7180a6182affea45`.
- A compilação final não apresentou avisos ou erros.
- Imports finais conferidos com `llvm-objdump -p`: somente Windows/UCRT,
  incluindo OpenGL32; nenhuma DLL adicional para acompanhar o programa.
- Inspeção de strings ASCII e UTF-16 confirmou a ausência do caminho
  pessoal usado na máquina de compilação.
- Pacote: `dist/emi-filter-windows-x86_64.zip`, com o executável na raiz,
  `LICENSE`, `THIRD_PARTY_NOTICES.md`, todos os textos de `licenses/` e os
  exemplos `assets/parametros-app-padrao.json` e `assets/parametros-10khz.json`.
- Tamanho do ZIP: 3.325.606 bytes.
- SHA-256 do ZIP: `120bf8923f418cb514ca0b20cc4621540860bef468dbad0e1ff1687d8c6c2579`.
- O teste de integridade do ZIP passou; o hash do executável extraído do
  pacote corresponde ao executável inspecionado acima. Os hashes dos dois
  exemplos extraídos também correspondem aos arquivos de origem.

O hash identifica esse executável. Uma nova compilação pode gerar outro
hash, mesmo quando os cálculos preservam os mesmos resultados.

## Caminhos opcionais

- `EMI_TOOLCHAIN_CACHE`: diretório do cache da toolchain.
- `EMI_WINDOWS_TARGET_DIR`: diretório temporário de compilação, padrão
  `/tmp/emi-rust-win-target`.
- `CARGO` e `RUSTUP`: caminhos para os executáveis, quando não estão em
  `~/.cargo/bin`.

Para inspecionar o resultado sem executar:

```sh
file dist/emi-filter-windows-x86_64.exe
llvm-objdump -p dist/emi-filter-windows-x86_64.exe
sha256sum dist/emi-filter-windows-x86_64.exe
```
