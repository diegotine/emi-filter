# Validação da entrega

## Publicação v1.3.0 — 23/10/2026

- 19 testes passaram com `cargo test --locked --offline`.
- `cargo fmt -- --check` e `cargo clippy --locked --offline -- -D warnings` passaram.
- Linux e Windows x86_64 compilados em release usando Cargo.lock.
- Linux executado: versão, cálculo padrão, exportação de parâmetros e CSV.
  O CSV até 1 MHz é idêntico ao usado na prévia aprovada.
- Captura real da nova interface Linux em `ui-preview.png`: painel lateral,
  legenda de L1 dentro do esquema, botões elevados, título/timestamp/legenda
  sobre a grade; eixo X logarítmico até 1 MHz e Y linear em dB.
- Série harmônica preservada até 300 kHz; equações do modelo preservadas.
  O teste C++ conserva sua faixa histórica para comparar o ponto exato de 10 kHz.
- Windows: PE32+ GUI x86_64 inspecionado; imports apenas do Windows/UCRT.
  Execução em Windows ainda não verificada.
- Executáveis e pacotes identificados com v1.3.0; hashes em SHA256SUMS.

## Build preliminar 1.3.0 (histórico) — 23/10/2026

- Painel azul de parâmetros à direita do diagrama, campos em coluna na
  largura lateral e gráfico aproveitando a altura disponível da janela.
- `cargo test --locked --offline`: 19 testes passaram.
- `cargo fmt -- --check` e `cargo clippy --locked --offline -- -D warnings`
  passaram. Núcleo do modelo, histórico e armazenamento preservados.
- Linux x86_64 compilado em release; `--version` informa 1.3.0.
- CSV padrão do executável conferido com `cmp`: idêntico ao da entrega 1.2.0.
- Interface renderizada em compositor de teste, janela de 1380 × 900:
  Cx1, L1 e sobreposição dos três gráficos. Capturas em `ui-preview.png`,
  `assets/ui-parametros-l1-v1.3.png` e `assets/ui-comparacao-v1.3.png`.
  Área do gráfico de atenuação aumentou aproximadamente de 155 para 330
  pixels de altura nessa janela.
- Nenhuma compilação Windows nesta versão, conforme pedido do usuário.

## Versão 1.2 — 20/10/2026

- 19 testes Rust: os 11 anteriores e cobertura dos rótulos de todos os campos;
  serialização de parâmetros adicionais e identificação do modelo; histórico
  circular e revisão do segundo slot; calendário/timestamps; Fourier do duty
  e do ringing contra integração numérica; pesos das derivadas; interpolação
  Z(f) sem extrapolação; bloqueio de não linearidade; acoplamento L2/L3 contra
  equivalentes modais ABCD; substituição de RLC/Cp por Z medido; fuga e parasitas
  de resistores; ressonância série amortecida; reciprocidade/energia dos novos C.
- Valores padrão e regressão contra o C++ preservados. Padrões pessoais e
  sessões têm versão/schema explícitos; JSONs planos antigos continuam aceitos.
- `cargo fmt -- --check`, `cargo clippy --locked -- -D warnings` e
  `git diff --check` verificados na entrega.
- Linux: interface renderizada em compositor de teste, painel principal/parasitas,
  grupo de L1 e sobreposição com três resultados em janela de 1380 × 900.
- Windows: compilação cruzada e inspeção dos imports do PE; execução em Windows
  ainda não verificada. Não alegar teste de execução do .exe no Linux.

A auditoria independente da versão 1.2 comparou 29,497 valores
contra MNA/NumPy/LAPACK: erro máximo 3.52e-09 dB.
Na regressão C++ foram 12,026 comparações, erro máximo
1.7e-11 dB. Matriz H: erro absoluto até 7,20e-12;
ondas/derivadas: erro relativo até 1,15e-12. Esses ensaios cobrem o padrão
e os casos lineares da auditoria anterior; as novas extensões específicas
são cobertas pelos testes descritos acima. CSV do executável entregue
idêntico ao do binário compilado. Dados e hashes no projeto original,
`verificacao_modelo_2026-10-20/versao_1p2/`.

## Versão 1.1 — 20/10/2026

- Onze testes Rust passaram, incluindo regressão dos padrões antigos;
  transferência complexa com conversão assimétrica; ramo de bypass e Cp
  comparados com ABCD independente; antirressonância; Parseval entre onda e
  espectro; derivadas conferidas por diferenças centrais; limite de trabalho
  da reconstrução temporal; CSVs com unidades e colunas verificadas.
- `cargo fmt -- --check`, `cargo clippy --locked -- -D warnings` e
  `git diff --check` passaram.
- Novos parasitas começam em zero e o bypass começa desativado; resultados
  padrão preservados. A nova curva total contabiliza a conversão entre modos.
- Linux: abas de ondas e fase renderizadas em compositor de teste. Onda e
  derivadas também exportadas pela CLI. Windows: compilação e inspeção PE;
  execução em Windows ainda não verificada.
- A auditoria independente completa e os dados antes/depois ficam na pasta
  `verificacao_modelo_2026-10-20` do projeto EMI Filter.

## Validação anterior — versão 1.0

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
