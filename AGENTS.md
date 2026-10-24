# Continuidade do aplicativo EMI Filter

## Entrega 1.3.0

- Alteração somente de layout: painel azul à direita do diagrama; gráfico
  abaixo aproveita a altura disponível, sem o teto antigo de 345 pontos.
- Editor usa uma coluna na largura lateral e três quando houver espaço.
- Título `EMI Filter v1.3.0 -- Made with Codex`; modelo e sessões preservados.
- Entrega atual compilada para Linux e Windows por autorização do usuário.
- Layout aprovado: legenda de L1 entre Δ/PE, botões sob esquema, título e
  timestamp dentro da grade; eixo Y linear em dB e X logarítmico até 1 MHz.
  Série harmônica continua até 300 kHz.

- Ler README.md, VALIDACAO.md e assets/modelo_filtro_120graus.json antes de
  alterar o modelo. Fontes em Rust; não introduzir Python no cálculo do app.
- Rede 0,5 Ω POR FASE em 60 Hz; modelo R+j2πfL com normalização em fref.
  Padrão R=0: Zfase=jf/120 Ω, ZDM=jf/60 Ω. CM total da rede 0,5 Ω em60Hz,
  ZCM=jf/120 Ω. Fonte 50 Ω/fase, DMtotal100 Ω, CMtotal1 kΩ. PE=0 Ω.
- Usar valores padrão e topologia do README. Ruído da direita para esquerda.
  Quadradas20kHz a120°, série ímpar até300kHz. Atraso da harmônica n é n120°.
  Alimentação60Hz com fase independente da portadora. Não sincronizar zeros.
- Atenuação é perda de inserção de tensão com as mesmas fontes e terminações
  sem filtro. Somar normas ao quadrado antes de converter para dB.
- Onda, dx/dt e d²x/dt² são bases separadas. Não inventar pesos para somá-las.
- Q40@1kHz é atual; Q60@10kHz pertence ao enrolamento anterior. Perdas AC
  constantes são aproximações; não são DCR para calcular aquecimento.
- src/engine.rs: nodal acoplado, perdas/parasitas, harmônicas e CSV.
  src/ui.rs: esquema editável, parâmetros e gráfico. Rótulos acompanham campos.
- Versão 1.1: preservar H complexa completa, inclusive termos cruzados, e
  reconstrução temporal com derivadas por (jω)^m. Conversão DM/CM é contada
  nas curvas totais de norma das fases; curvas co-modais continuam disponíveis.
- Novos campos ideais em zero: cx3_bypass_nf/esr_ohm/esl_nh, l2_cp_pf e
  l3_cp_pf. Bypass é ramo RLC independente em paralelo com Cx3; não fica em
  série com o Rd de Cx2. Não confundir parasitas lineares com C(V)/L(I).
- JSONs antigos carregam com padrões ideais. CSV de atenuação tem 11 colunas;
  --transfer exporta H/H0 complexos e --waveforms exporta ondas e derivadas.
- JSON legado do app é plano com unidades nos nomes; a versão 1.2 adiciona
  extra e envelope de sessão. O JSON histórico dos estudos é outro schema.
- cargo test --locked; cargo fmt -- --check; cargo clippy --locked -- -D warnings.
  Builds: build-linux.sh e build-windows.sh. Um executável para cada sistema.
- Windows compilado/inspecionado; execução ainda não verificada.
- Artefatos incluem versão no nome: emi-filter-v1.3.0-linux-x86_64 e
  emi-filter-v1.3.0-windows-x86_64.exe. README contém mudanças e link à v1.0.0.
- Preservar o padrão validado de 20 kHz. O exemplo assets/parametros-10khz.json
  representa a frequência observada por captação de campo próximo; não é
  medição da amplitude conduzida ou do sincronismo entre servos.
- Publicação: código, documentação e recursos no Git; executáveis e pacotes
  em Releases. Não versionar target/, dist/ ou configurações pessoais.

## Continuidade da versão 1.2.0

- Título: `EMI Filter v1.2.0 -- Made with Codex`. O título identifica a entrega;
  não é garantia sobre a autoria de futuras edições externas.
- Interface agrupa valor principal e adicionais por componente; nomes no
  esquema abrem o grupo. `parameter_editor.rs` mantém rótulos/unidades/busca.
- Parâmetros adicionais em `Parameters.extra`, módulo `extended.rs`. Padrões
  físicos anteriores preservados. Parasitas não medidos em zero são ideais;
  limites e leituras avulsas desconhecidos são None, não medições em zero.
- Solver incorpora k L2/L3, C concentrados de L1, fuga, L/C dos resistores,
  duty, pesos opcionais e ringing desativado até obter duração/τ. Fórmulas e
  convenções no README. C(V)/R(V) não nulos devem bloquear cálculo linear.
- Z(f) escalar ativado substitui RLC/Cp, sem extrapolação. Não usar em L1 ou
  com k L2/L3 não nulo. A interpolação não identifica causalidade.
- Cadastro Q/DCR/|Z| e limites não altera parâmetros efetivos automaticamente.
  Não confundir capacitância equivalente do LCR com Cp nem Rs AC com DCR.
- `history.rs`: máximo 3, ordem cronológica de inserção; quarto desloca 2→1,
  3→2. Revisão do slot 2: [A,B,C]→[B,Bnovo,C]. Carregar um slot não recalcula.
- Sobreposição: cores por posição, DM contínuo/CM tracejado, escalas comuns.
  Timestamps em UTC−03 com milissegundos, mantidos na sessão.
- `storage.rs`: JSON GUI em envelope schema2/model_id
  `emi-two-conductor-linear-v2`, com parâmetros, histórico e versão. Aceita
  legado plano; rejeita modelo/schema desconhecidos. CLI aceita ambos.
- Padrões pessoais são explícitos via Salvar padrões e lidos na abertura;
  `EMI_DEFAULTS_PATH` é útil para testes isolados. Não sobrescrever dados do usuário.
- 19 testes; a validação e a regressão independente precisam continuar passando.
- Entrega local ativa: `EMI-Filter-entrega-v1.2.0` na pasta do projeto original.
