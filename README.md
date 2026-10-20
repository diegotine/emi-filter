# EMI Filter

Aplicativo nativo em Rust para alterar o filtro EMI e calcular sua perda de
inserção nos modos diferencial (DM) e comum (CM). Valores editáveis sobre o
esquema, gráfico com cursor, análise harmônica e exportação JSON/CSV.

![Interface do EMI Filter](ui-preview.png)

A interface clara, azul e com grade foi inspirada no estilo do PicoScope 7.
Este é um projeto independente, sem vínculo com a Pico Technology.

## Executar

Baixe os executáveis em [Releases](https://github.com/diegotine/emi-filter/releases/latest):

- Linux x86_64, glibc 2.43 ou mais recente: `emi-filter-linux-x86_64` ou o pacote `.tar.gz`.
- Windows 10/11 x86_64: `emi-filter-windows-x86_64.exe` ou o pacote `.zip`.

No Linux, dê permissão de execução com `chmod +x emi-filter-linux-x86_64` e
abra o arquivo. No Windows, abra o `.exe`. Cada executável contém o cálculo
e os recursos usados pela interface.

Cada sistema tem seu próprio executável. Os recursos e o cálculo ficam dentro
dele; não precisa instalar Rust, Python ou abrir os arquivos do projeto.
O Linux usa as bibliotecas gráficas do sistema; o Windows usa as bibliotecas
do próprio Windows. Os arquivos JSON/CSV só são necessários quando você deseja
salvar parâmetros ou exportar os resultados.
Para Linux com glibc anterior à 2.43, compile a partir dos fontes no seu sistema.

## Usar

Altere os valores diretamente no esquema. Abaixo dele ficam os parâmetros da
rede, da fonte de ruído e as perdas e capacitâncias parasitas. Clique em
**Calcular atenuação** depois de mudar os valores, ou use **Ctrl+Enter**.
As curvas DM e CM representam
a varredura contínua; os pontos representam as harmônicas das duas quadradas.
O cursor mostra a frequência e os valores de atenuação.

Os campos de alimentação, fase de 60 Hz e corrente registram as condições de
operação. No modelo linear, não alteram a transferência do ruído. As fases das
quadradas são independentes das fases da alimentação. Alterar a amplitude do
ruído também não altera uma razão de atenuação em um circuito linear.

## Modelo padrão

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

- [`assets/parametros-app-padrao.json`](assets/parametros-app-padrao.json):
  ensaio original de duas quadradas de 20 kHz, usado na validação.
- [`assets/parametros-10khz.json`](assets/parametros-10khz.json): mesmo circuito,
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

Os resultados de validação estão em [`VALIDACAO.md`](VALIDACAO.md).
O executável Windows foi compilado e inspecionado, mas ainda não foi executado
em Windows. Os arquivos SHA-256 de cada versão acompanham os downloads.

## Licença

Código sob [licença MIT](LICENSE). Avisos de bibliotecas, fontes e runtimes
incorporados em [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
