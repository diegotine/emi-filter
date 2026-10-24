# Manual e referência do EMI Filter

Referência consolidada das funções herdadas e do modelo. Para as alterações
da entrega atual, veja o [README](../README.md). A varredura padrão atual vai
até 1 MHz; a série harmônica permanece limitada a 300 kHz.

## Usar

Altere os valores diretamente no esquema. À direita dele ficam os parâmetros da
rede, da fonte de ruído e as perdas e capacitâncias parasitas. Clique em
**Calcular atenuação** depois de mudar os valores, ou use **Ctrl+Enter**.
As curvas DM e CM representam
a varredura contínua; os pontos representam as harmônicas das duas quadradas.
O cursor mostra a frequência e os valores de atenuação.

Os campos de alimentação, fase de 60 Hz e corrente registram as condições de
operação. No modelo linear, não alteram a transferência do ruído. As fases das
quadradas são independentes das fases da alimentação. Alterar a amplitude do
ruído também não altera uma razão de atenuação em um circuito linear.

## Novidades da versão 1.2

A janela inicial tem 1380 × 900 e a barra de título identifica
**EMI Filter v1.2.0 -- Made with Codex**. Clique no nome de um componente no
esquema ou selecione-o em **Componente / parâmetros**: seu valor principal,
perdas e parasitas ficam juntos no painel compacto. A busca encontra campos
em todos os grupos. As seções de medições/limites e Z(f) podem ser expandidas.

**Salvar padrões** guarda os valores atuais para as próximas aberturas.
**Carregar padrões** recupera esses valores; **Padrões de fábrica**, na barra
lateral, recupera a configuração original. Os parasitas não medidos mantêm
zero como aproximação ideal; R de fuga=0 representa infinito. Leituras e
limites ainda não informados permanecem em branco, sem inventar medições.
Padrões pessoais ficam em `$XDG_CONFIG_HOME/emi-filter/padroes.json`, ou
`~/.config/emi-filter/padroes.json`; no Windows, `%APPDATA%/emi-filter/padroes.json`.
`EMI_DEFAULTS_PATH` permite escolher outro arquivo.

### Três gráficos para comparar

- A abertura já calcula o **Gráfico 1**. Mude valores e use **Calcular atenuação**
  para gerar o próximo. Um botão de posição vazia gera o próximo gráfico;
  posições ainda fora da sequência ficam desativadas.
- Os botões **Gráfico 1**, **Gráfico 2** e **Gráfico 3** carregam os valores e o
  resultado daquela posição. Cada cálculo guarda uma cópia dos parâmetros e
  seu timestamp, com milissegundos e fuso UTC−03 (São Paulo).
- Após três resultados, calcular a partir do mais recente descarta o primeiro:
  o segundo vira primeiro e o terceiro vira segundo.
- Ao selecionar e editar o segundo, **Calcular atenuação** mantém a revisão no
  segundo: `[A, B, C] → [B, Bnovo, C]`. Repetir a edição preserva a revisão
  anterior à esquerda. No primeiro slot, a revisão substitui o próprio slot,
  pois não existe posição anterior. **Novo gráfico** força uma adição ao final.
- **Sobrepor**, abaixo dos botões, compara os resultados: azul, laranja e verde
  identificam as posições 1, 2 e 3. Nas curvas espectrais, DM é contínuo e CM
  tracejado. Também há sobreposição de fase e de onda filtrada. A sobreposição
  usa escalas comuns, incluindo a união das frequências disponíveis; a opção de
  escala dB manual continua disponível.
- **Salvar** em Arquivos grava a sessão: valores em edição, os três conjuntos
  calculados e timestamps. **Abrir** reconstrói o histórico. CSV e tabela
  continuam correspondendo ao gráfico selecionado.

Os JSONs salvos pela interface incluem identificação do app, sua versão,
`model_id=emi-two-conductor-linear-v2` e `schema_version=2`. O app rejeita
outros identificadores de modelo/schema e continua lendo configurações planas
antigas. A CLI aceita os dois formatos. `--write-defaults` preserva o formato
plano para ferramentas existentes. O JSON histórico dos estudos C++ permanece
outro formato, sem alterações.

### Parâmetros adicionais e efeito no cálculo

Além dos 53 campos anteriores, o painel reúne:

- Acoplamento assinado L2↔L3, `M=k√(L2L3)`; padrão k=0.
- Capacitância entre os potenciais médios dos enrolamentos de L1 e capacitância
  total desses enrolamentos ao núcleo/PE. São aproximações concentradas: no
  segundo caso, o núcleo é equipotencial ao PE e a capacitância se divide
  igualmente. Ambas começam em zero; não substituem uma matriz medida do núcleo.
- Resistência de fuga dos capacitores; L série e C paralelo de Rd/R1/R2.
- Duty da quadrada e soma opcional das bases, com
  `W(jω)=c0+jωc1−ω²c2`. Padrão: soma desativada, pesos efetivos 1, 0 e 0.
  Os pesos não representam novas medições.
- Exemplo de ringing periódico: `A·escala·exp(−t/τ)·sin(2πfr·t)` até a duração
  do pacote, repetido no intervalo definido. Frequência 50 kHz, intervalo
  0,1 ms e escala 5 são o exemplo discutido. Começa desativado; duração e τ
  precisam ser informados para ativá-lo. Atraso entre fases refere-se ao período
  de repetição. A média DC é omitida tanto no ringing quanto em duty diferente
  de 50%; a série AC e as derivadas continuam truncadas em fmax.
- Tabelas de Re/Im de Z(f) por componente, com ativação explícita. Substituem
  o ramo RLC/Cp completo do componente (Rd continua externo a Cx2).
  Interpolação linear de Re/Im em log(f), sem extrapolação. Todos os pontos
  usados na curva e nas harmônicas precisam estar dentro da faixa medida.
  Uma tabela escalar não representa L1 nem L2/L3 acoplados. Interpolação
  espectral não garante causalidade ou precisão fora dos pontos medidos.
- Cadastro separado de Q, |Z|, DCR, frequência/condições do ensaio, capacitância
  equivalente do LCR, tolerância, tensões, correntes, potência, saturação,
  temperatura e notas. Esses registros não substituem silenciosamente os
  valores efetivos usados no cálculo nem simulam aquecimento/saturação.

Coeficientes `C(V)=C0(1+αV+βV²)` e `R(V)=R0(1+αV+βV²)` podem ser registrados.
O solver continua linear: valores não nulos **bloqueiam o cálculo**, em vez de
serem ignorados ou de produzir H2/H3 fictícias. Podem ser guardados na sessão
para um futuro solver não linear; os padrões para cálculo precisam ser válidos.
A planilha não foi importada automaticamente nem trocou os indutores de 1,1 µH
pelas bobinas alternativas do amplificador.

## Modelo padrão

### Novidades da versão 1.1

As abas **Fase de H** e **Formas de onda** usam os fasores complexos do
solver. As ondas antes/depois ficam na mesma escala, selecionando fase A,
fase B, DM ou CM; as derivadas são exibidas em V/s e V/s². São dois períodos
do ruído periódico, sem somar os 60 Hz da alimentação. A série truncada,
com ≥16 amostras por ciclo da maior harmônica, não representa bordas ideais
infinitamente rápidas. Séries excessivamente longas mantêm a análise espectral
e deixam a reconstrução temporal indisponível, evitando aliasing/travamento.

**Incluir conversão DM↔CM** fica ligado por padrão. Cada curva modal passa
a medir a norma das duas fases para aquela entrada pura, contando também o
ruído convertido. Desligue para ver apenas o próprio modo. No circuito
simétrico padrão, os dois resultados coincidem. A referência continua sendo
a mesma rede sem filtro, não potência ativa absorvida.

Na aba **Bypass e autorressonância**, um ramo C/ESR/ESL independente pode ser
colocado em paralelo com Cx3, e Cp pode ser adicionado a L2 e L3. O solver
inclui as autorressonâncias e antirressonâncias dessa rede. Os cinco campos
novos começam em zero (ideal; bypass desativado). Valores definidos de L/C,
Q e perdas permanecem os mesmos. JSONs anteriores carregam com esses padrões.
Capacitores adicionados entre fases precisam de classe X2 na montagem.

Os botões **Exportar H complexa** e **Exportar ondas** salvam arquivos irmãos
do CSV indicado. A matriz modal completa preserva termos cruzados e Re/Im,
com H relativo à fonte em aberto; H0 sem filtro também é exportado. Fases
no CSV são principais; a aba mostra a fase desenrolada dos termos diretos.
O CSV de atenuação preserva as nove colunas anteriores e acrescenta duas:
`IL_DM_total_dB` e `IL_CM_total_dB`. Os demais tipos de linha deixam-nas vazias.

O motor continua linear: parasitas não criam H2/H3 novas. Não linearidade
C(V), L(I), saturação e retificadores ainda exigem
modelos próprios. As harmônicas da quadrada pertencem à fonte.

```sh
dist/emi-filter-v1.3.0-linux-x86_64 --transfer parametros.json transferencia.csv
dist/emi-filter-v1.3.0-linux-x86_64 --waveforms parametros.json ondas.csv
```

Rede → Cx1 → L1 CM → [Rd–Cx2 entre fases; Cy1/Cy2 ao PE] → L2/L3 → Cx3 → servos.
Ruído se propaga da direita para a esquerda.

| Componente | Valor |
|---|---:|
| Cx1 / Cx2 / Cx3 | 1,5 / 0,47 / 0,33 µF |
| Cy1 / Cy2 | 3,3 nF cada |
| Rd | 4,7 Ω |
| R1 / R2 de descarga | 150 kΩ cada, em série |
| L1, medição série aditiva | 1,045 mH |
| L1, medição série oposta | 8,8 µH |
| L1, DCR por enrolamento | 22 mΩ |
| L1, Q CM | 40 a 1 kHz |
| L2 / L3 | 1,1 µH cada |
| Perda AC de L2 / L3 | 0,121642098 Ω cada |

Os 0,5 Ω da rede são a **referência adotada por fase em 60 Hz**.
O programa calcula a indutância a partir dessa hipótese:

```text
Lfase = sqrt(|Zref|² − Rfase²) / (2π fref)
Zfase(f) = Rfase + j 2π f Lfase
ZDM(f) = 2 Zfase(f)
```

Com os padrões R=0 e fref=60 Hz: `Zfase=j·f/120 Ω` e `ZDM=j·f/60 Ω`.
O modo comum da rede é uma porta independente com 0,5 Ω a 60 Hz, seguindo
a mesma hipótese indutiva: `ZCM=j·f/120 Ω`. A fonte tem 50 Ω por fase,
equivalentes a 100 Ω DM, e 1 kΩ na porta CM completa. PE=0 Ω.

O cálculo usa análise nodal complexa de ambos os condutores, incluindo
acoplamento em L1, conversão DM/CM quando houver assimetrias e PE não ideal
quando seus parâmetros forem editados. A matriz de impedância em cada porta é:

```text
Zaa = Zbb = ZCM + ZDM/4
Zab = Zba = ZCM − ZDM/4
```

Excitação padrão: duas quadradas de 20 kHz com atraso relativo de 120°,
fase inicial de 37°, série ímpar até 300 kHz. A harmônica n recebe atraso
n×120°. As bases x, dx/dt e d²x/dt² são integradas separadamente.

### Configurações de exemplo

- [`assets/parametros-app-padrao.json`](../assets/parametros-app-padrao.json):
  ensaio original de duas quadradas de 20 kHz, usado na validação.
- [`assets/parametros-10khz.json`](../assets/parametros-10khz.json): mesmo circuito,
  com a frequência das quadradas ajustada para 10 kHz.

Para abrir um exemplo, informe seu caminho no campo **Configuração JSON** e
clique em **Abrir**. O resultado é calculado ao carregar o arquivo.

Foi observada uma componente de aproximadamente 10 kHz ao aproximar uma
ponteira de osciloscópio do inversor, sem conexão elétrica direta. Essa
observação orienta a frequência do segundo exemplo. As duas quadradas e seu
atraso de 120° continuam sendo hipóteses do ensaio; a amplitude é normalizada.

```text
IL(f) = 20 log10(|Vsem filtro| / |Vcom filtro|)
ILconjunta = 10 log10(Σ|Vfase,sem|² / Σ|Vfase,com|²)
```

As mesmas fontes e terminações são usadas nas duas condições. A integração
soma os módulos ao quadrado antes de converter para dB. Valores negativos
indicam reforço de tensão relativo à referência. As derivadas multiplicam
cada componente espectral por `(j·2πf)^m`; não são harmônicas adicionais.

Perdas AC são mantidas constantes por falta de outras medições. ESR, ESL e
capacitância dos enrolamentos não medidas começam em zero como aproximação
ideal. O modelo não calcula saturação ou aquecimento, nem os retificadores e
PWM reais dos servos. Resistência AC obtida por Q não é DCR para calcular
perdas a 5 A. A referência de Q=60 a 10 kHz pertence ao enrolamento anterior.

## Compilar

Rust 1.88 ou mais recente e as dependências nativas de X11/Wayland/OpenGL são necessários para
compilar no Linux. Use `./build-linux.sh`. O `Cargo.lock` fixa as versões.
O script `build-windows.sh` faz a compilação cruzada com LLVM-MinGW.

```sh
git clone https://github.com/diegotine/emi-filter.git
cd emi-filter
./build-linux.sh
```

As compilações ficam em `dist/`. Os scripts removem o caminho pessoal de
compilação dos binários com `--remap-path-prefix`.

Para conferir os padrões sem abrir uma janela:

```sh
cargo run --release -- --calculate-defaults
cargo run --release -- --write-defaults modelo_usuario.json
cargo run --release -- --calculate modelo_usuario.json atenuacao.csv
cargo run --release -- --calculate assets/parametros-10khz.json atenuacao-10khz.csv
cargo test --locked
```

O PDF e o modelo original completo foram preservados em `assets/`.
O JSON editável do aplicativo é uma lista plana de parâmetros com as unidades
nos nomes das chaves; ele tem formato próprio, diferente do registro histórico
completo do modelo original. Chaves desconhecidas são rejeitadas.

Os resultados de validação estão em [`VALIDACAO.md`](../VALIDACAO.md).
O executável Windows foi compilado e inspecionado, mas ainda não foi executado
em Windows. Os arquivos SHA-256 de cada versão acompanham os downloads.

## Licença

Código sob [licença MIT](../LICENSE). Avisos de bibliotecas, fontes e runtimes
incorporados em [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).
