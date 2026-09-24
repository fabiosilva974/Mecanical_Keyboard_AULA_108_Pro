# Teclado Mecânico AULA F108 Pro - Configuração no Linux

[![English](https://img.shields.io/badge/English-README.md-blue)](README.md)
[![Interface Gráfica Linux](https://img.shields.io/badge/GUI-f108--pro--gui-purple.svg)](f108-pro-gui)
[![Driver em Rust](https://img.shields.io/badge/Driver-Rust%20CLI-orange.svg)](f108-pro-rust)

> 🚀 **NOVIDADE: Suíte Nativa para Linux (Sem necessidade de Wine!)**
> - **[Interface Gráfica Nativa (`f108-pro-gui`)](f108-pro-gui)**: Interface moderna em WebKitGTK, canvas interativo de 104 teclas, compositor de comandos CLI retrátil em tempo real, 20 modos RGB, pintura por tecla e gerenciador do visor TFT LCD.
> - **[Driver Nativo em Rust (`f108-pro-rust`)](f108-pro-rust)**: Driver CLI independente de alta performance para Linux com suporte a USB HID e envio de imagens para o visor TFT LCD.
>
> *(O guia legado de configuração via Wine está preservado abaixo para referência).*

Este repositório fornece ferramentas, drivers e guias passo a passo sobre como configurar o teclado mecânico **AULA F108 Pro** no Linux de forma nativa ou via Wine.

O teclado AULA F108 Pro é normalmente identificado pelo sistema como `Bus XXX Device YYY: ID 0c45:800a Microdia Vivitar Vivicam3350B` no `lsusb`.

**Curiosidade:** A fabricante *Sonix/Microdia* (`Vendor ID: 0c45`) é a principal fornecedora de chips para a grande maioria dos teclados mecânicos modernos (incluindo AULA, Redragon, Royal Kludge). O nome "Vivitar Vivicam" é apenas um conflito de nomes no banco de dados de USB do Linux, que acha que esse ID pertence a uma câmera velha, mas na verdade é o controlador do seu teclado.

Aqui está o guia exato de como fazê-lo funcionar, utilizando o ID confirmado `0c45:800a`.

---

## Passo Zero: Como descobrir o ID exato de qualquer teclado usando um Teclado Virtual

Se você estiver configurando outro teclado ou em outra máquina e precisar confirmar o ID exato dele, o método mais garantido é comparar a lista de dispositivos USB conectados antes e depois de remover o cabo. Como você ficará sem teclado para apertar a tecla "Enter", o truque é usar um teclado na tela:

1. Abra um teclado virtual no seu Linux (no Mint/Ubuntu, você pode procurar por **Onboard** no menu iniciar, ou ativar o **Teclado Virtual** nas Configurações de Acessibilidade).
2. Abra o terminal e digite `lsusb` (mas não aperte Enter ainda).
3. Com o teclado virtual já aberto na tela, use o mouse e clique no botão **Enter / Return** do teclado virtual. Salve ou anote o resultado que aparecer no terminal.
4. Digite `lsusb` de novo no terminal (não aperte Enter).
5. Desconecte o cabo do seu teclado.
6. Volte a clicar no botão **Enter** do teclado virtual com o mouse.
7. Compare as duas listas! A linha inteira que **sumiu** na segunda listagem é exatamente o seu teclado. Guarde o ID (os números no formato `xxxx:yyyy`, por exemplo `0c45:800a`).

## Passo 0.5: Instalar o Software no Wine

Antes de lidarmos com as permissões avançadas do sistema, certifique-se de que o software oficial está instalado no seu Linux através do Wine:

1. Baixe o instalador oficial na [Página de Download AULA Gaming](https://www.aulagaming.com/pages/download?srsltid=AU7gw4X016m2J_EuRJDXVzFc_UaQqpcjpYPqa6QOKvJNLoxv8DQ099PS) ou use o instalador disponibilizado neste repositório (caso disponível).
2. Extraia o instalador que vier dentro da pasta compactada `.zip` ou `.rar`.
3. Clique com o botão direito no instalador `.exe` e escolha **"Abrir com o Wine"** (ou algo como "Carregador de Aplicativos Windows Wine"). 
   
   **Alternativa via Terminal (Linha de Comando):**
   Se você prefere instalar pelo terminal, basta abrir a pasta, extrair o arquivo e executar via Wine:
   ```bash
   wine NomeDoInstalador.exe
   ```

   *(Atenção: Se durante a execução o Wine pedir permissão para baixar ou instalar pacotes extras como Mono ou Gecko, você pode aceitar todos).*
4. Basta avançar na instalação (botão Next) até finalizar, exatamente igual no Windows.

## Passo 1: Criar a regra de permissão do `udev`

Abra o seu terminal e execute o editor `nano` como administrador para criar um arquivo dedicado ao seu teclado:

```bash
sudo nano /etc/udev/rules.d/99-aula-f108-pro.rules
```

Cole o seguinte conteúdo dentro do arquivo:

```udev
# Regra udev para liberar acesso ao software do AULA F108 Pro via Wine (Cabo USB)
SUBSYSTEM=="usb", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="800a", MODE="0666", ENV{ID_GPHOTO2}="" 
KERNEL=="hidraw*", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="800a", MODE="0666", ENV{ID_GPHOTO2}=""

# Regra udev para liberar acesso ao software via Dongle 2.4G sem fio
SUBSYSTEM=="usb", ATTRS{idVendor}=="05ac", ATTRS{idProduct}=="024f", MODE="0666"
KERNEL=="hidraw*", ATTRS{idVendor}=="05ac", ATTRS{idProduct}=="024f", MODE="0666"
```

> **Entendendo a regra gerada:**
> - `SUBSYSTEM=="usb"` e `KERNEL=="hidraw*"`: Dizem ao Linux que esta regra se aplica à conexão USB e à interface de comunicação "bruta" (raw HID) do dispositivo. O software precisa desse acesso direto para poder gravar macros e modificar as cores RGB.
> - `ATTRS{idVendor}=="0c45"` e `ATTRS{idProduct}=="800a"`: É aqui que aplicamos os dados descobertos com o comando `lsusb`. Isso serve como um "filtro" para garantir que a permissão especial seja dada **apenas** ao seu teclado AULA, mantendo a segurança do resto dos seus dispositivos USB intacta.
> - `MODE="0666"`: Concede permissão de leitura e escrita (Read/Write) para o dispositivo. Como o Wine executa o software do teclado usando a sua conta de usuário comum (e não como administrador/root), sem o `0666` o programa seria bloqueado pelo Linux de tentar alterar o hardware.
> - `ENV{ID_GPHOTO2}=""`: Remove a classificação de "câmera digital" que o sistema estava impondo erroneamente a esse dispositivo. Sem isso, alguns gerenciadores de USB do Linux e do Wine bloqueiam a comunicação HID.

Para salvar e sair do `nano`:
1. Pressione `Ctrl+O` e dê `Enter` para salvar.
2. Pressione `Ctrl+X` para fechar.

## Passo 2: Recarregar as regras no sistema

Para que o Linux da sua máquina leia esse arquivo agora mesmo sem precisar reiniciar o computador, rode:

```bash
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Passo 2.5: Configurar o Registro do Wine

Por padrão, o Wine pode bloquear dispositivos de entrada para evitar conflitos com o sistema Linux. Para forçar o Wine a ler os dispositivos USB Brutos (raw HID), aplique estas configurações no registro rodando os comandos abaixo no terminal:

```bash
wine reg add "HKLM\System\CurrentControlSet\Services\WineBus" /v "Enable SDL" /t REG_DWORD /d 0 /f
wine reg add "HKLM\System\CurrentControlSet\Services\WineBus" /v "DisableInput" /t REG_DWORD /d 0 /f
```
Após isso, reinicie os serviços do Wine:
```bash
wineserver -k
```

## Passo 3: Executar o Software da AULA

Com o software oficial já instalado através do Wine, você deve iniciá-lo. O comando exato dependerá de onde o Wine colocou a pasta, mas costuma ser o seguinte:

```bash
wine ~/.wine/drive_c/ProgramFilesx86/AULA_F108Pro/DeviceDriver.exe 
```
*(Caso o atalho tenha sido criado na sua Área de Trabalho, você também pode simplesmente clicar duas vezes nele se o Wine estiver configurado para isso).*

---

## Bônus: Como arrumar o nome do teclado no `lsusb`

Se você quiser corrigir o "erro estético" do Linux e fazer com que o comando `lsusb` exiba o nome "AULA F108 Pro" corretamente no lugar de "Vivitar Vivicam3350B", você pode editar o banco de dados de nomes USB do seu sistema.

1. Abra o arquivo de banco de dados USB (`usb.ids`) como administrador:
```bash
sudo nano /usr/share/hwdata/usb.ids
```
*(Nota: dependendo da sua distribuição, o arquivo pode ficar em `/usr/share/misc/usb.ids`)*

2. No nano, pressione `Ctrl+W` para abrir a pesquisa, digite `800a` e dê `Enter`.
3. Ele vai pular diretamente para a seguinte linha:
   `	800a  Vivitar Vivicam3350B`
4. Basta apagar o nome da câmera e digitar o nome do seu teclado, deixando a linha assim:
   `	800a  AULA F108 Pro`
5. Salve e saia do nano (`Ctrl+O`, `Enter`, `Ctrl+X`).

Pronto! Na próxima vez que você rodar o comando `lsusb`, a sua lista vai exibir orgulhosamente o nome correto do seu teclado!
