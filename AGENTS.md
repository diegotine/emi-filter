# Continuidade do aplicativo EMI Filter

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
- JSON do app é plano com unidades nos nomes. O JSON histórico é outro schema.
- cargo test --locked; cargo fmt -- --check; cargo clippy --locked -- -D warnings.
  Builds: build-linux.sh e build-windows.sh. Um executável para cada sistema.
- Windows compilado/inspecionado; execução ainda não verificada.
- Preservar o padrão validado de 20 kHz. O exemplo assets/parametros-10khz.json
  representa a frequência observada por captação de campo próximo; não é
  medição da amplitude conduzida ou do sincronismo entre servos.
- Publicação: código, documentação e recursos no Git; executáveis e pacotes
  em Releases. Não versionar target/, dist/ ou configurações pessoais.
