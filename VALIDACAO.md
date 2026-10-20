# Validação da entrega

20/10/2026. Compilador Rust 1.98.0. Dependências fixadas em `Cargo.lock`.

- `cargo test --locked`: sete testes do modelo passaram.
- Regressão contra o cálculo C++ salvo: RMS, derivadas, DM a 10 kHz e
  harmônicas, com tolerância de 0,0000001 dB.
- Verificação independente por matrizes ABCD, incluindo PE com resistência
  e indutância; teste de assimetrias e parasitas no modelo nodal completo.
- Testes da referência de 0,5 Ω **por fase a 60 Hz**, crescimento indutivo
  com frequência e independência das fases da portadora e da alimentação.
- Modos ausentes na excitação aparecem como `—`, sem valores espúrios.
- `cargo fmt -- --check` e `cargo clippy --locked -- -D warnings` passaram.
- Linux: aplicação abriu com o esquema e o gráfico completos. Edição de
  componente e salvamento JSON verificados. Recálculo por Ctrl+Enter
  verificado com valor alterado. Botão de cálculo e exportação CSV também
  verificados numa sessão gráfica de teste separada (Weston/Xwayland).
- CSV padrão exportado preservado em `assets/atenuacao-padrao.csv`.
- Windows: compilação cruzada concluída e formato/imports inspecionados.
  Não foi executado em Windows; ver `WINDOWS_BUILD.md`.

## Resultados padrão

| Base | Conjunta / dB | DM / dB | CM / dB |
|---|---:|---:|---:|
| x(t) | 11,36742799 | 28,27482694 | −0,45962463 |
| dx/dt | 11,08249387 | 34,69871394 | 7,06191894 |
| d²x/dt² | 19,11438005 | 47,28962425 | 16,43291829 |

DM a 10 kHz: 21,34955763 dB. Esses resultados são estimativas do modelo
com as hipóteses documentadas, não medições da montagem.

## Preparação para publicação

Os executáveis foram recompilados com remapeamento dos caminhos locais de
compilação. O binário Linux requer glibc 2.43 ou mais recente, conforme seus
símbolos dinâmicos. As fontes e o modelo padrão foram preservados.

O exemplo `assets/parametros-10khz.json` mantém o circuito e ajusta a frequência
das quadradas para 10 kHz. Atenuações RMS conjuntas calculadas nesse ensaio:
13,32097844 dB para x(t), 13,29512633 dB para dx/dt e 20,19105011 dB para
d²x/dt². A observação por captação de campo próximo é registrada no README;
não fornece a amplitude conduzida nem o sincronismo real dos três servos.
