# RosaClient ユーザーマニュアル

**zinro.net**（人狼ゲームプラットフォーム）の非公式 TUI クライアント。ターミナルからチャットを閲覧・発言できます。

> **初めてですか？** [インストール]から始めれば、数分で動きます。

---

## 1. 概要

### Rosa は何をするツールか

Rosa は [zinro.net](https://zinro.net) 向けの軽量な **ターミナルユーザーインターフェース（TUI）クライアント** です。ブラウザを開かずに、ターミナルからチャットの閲覧と発言ができます。

主な機能：

- **2秒ごとにサーバーをポーリング**（カスタマイズ可能）してチャットメッセージとプレイヤー情報を取得・表示 (`src/main.rs:711`, `src/api.rs:18`)
- **全体チャット（ALL）へのメッセージ送信** (`src/api.rs:223`)
- **参加者一覧と部屋情報**を表示—部屋名、シーン、日付、定員 (`src/ui.rs:350`)
- **Vim風のキーバインド**と4パネルレイアウト（Explorer / Chat / Context / Terminal）で操作
- **送信中のメッセージを「sending…」として一時表示** (`src/main.rs:564`)
- **接続状態（online / offline）をリアルタイム表示**

### Rosa を使うメリット

- **ターミナルだけで完結** — ブラウザは不要
- **キーボードだけで高速操作** — Vim風のコマンド体系
- **軽量で単純** — 依存関係は最小限
- **オープンソース** — コードを確認・改造できます

### 免責事項

Rosa は zinro.net の **非公式クライアント** です。自己責任でご利用ください。

---

## 2. 必要な環境

### 対応OS

Rosa は Rust で書かれており、Rust と `crossterm`（ターミナル制御ライブラリ）が動く環境で動作します。

- Windows
- Linux  
- macOS

### 必須の依存関係

ビルド・実行には以下が必要です。

- **Rust ツールチェーン**（`cargo` を含む）
  - Edition 2024 (`Cargo.toml:4`)。Stable Rust を推奨します。
- **インターネット接続**（`https://zinro.net` へ通信するため）
- **有効なセッションキー**（後述のセットアップで説明）

### 主要ライブラリ

| クレート | 用途 |
|-----------|------|
| `tokio` | 非同期ランタイム |
| `reqwest` | HTTP クライアント |
| `ratatui` / `crossterm` | TUI 描画・キー入力 |
| `serde` / `serde_json` | JSON パース |
| `dotenvy` | `.env` ファイル読み込み |
| `anyhow` | エラー処理 |
| `tracing` / `tracing-subscriber` | ログ記録 |

### 推奨環境

- **Nerd Fonts 対応ターミナルフォント**（推奨）
  - Rosa は Nerd Fonts の Unicode グリフ（👤 など）を使用しています (`src/ui.rs`)。対応フォントがないと □ や ? で表示されます。
- **True Color (24ビットカラー)対応ターミナル**（推奨）
  - RGB 色指定を使用しています (`src/ui.rs:17+`)。対応ターミナルなら美しく表示されます。

---

## 3. インストール

Rosa には配布用のインストーラやパッケージはありません。**ソースコードからビルド**します。

### ステップ1: Rust をインストール

Rust がない場合は、公式インストーラから入手します。

**→ <https://rustup.rs>** （Windows / macOS / Linux 共通）

インストール後、ターミナルを開き直して確認します。

```bash
cargo --version
```

`cargo 1.xx.x ...` と表示されれば成功です。

### ステップ2: ソースコードを入手

Rosa のリポジトリをクローンまたはダウンロードして、プロジェクトディレクトリに移動します。

**Linux / macOS:**

```bash
cd /path/to/rosa
```

**Windows (PowerShell):**

```powershell
cd C:\path\to\rosa
```

### ステップ3: ビルド

リリースビルドを作成します。

```bash
cargo build --release
```

成功すると実行ファイルが生成されます。

- **Linux / macOS:** `target/release/rosa`
- **Windows:** `target/release/rosa.exe`

### プラットフォーム別の注意

`cargo` コマンド自体はどのOS でも同じです。違いは以下だけです。

- パス区切り文字（Windows: `\` / Unix: `/`）
- 実行ファイルの拡張子（Windows: `.exe` / Unix: なし）
- ターミナル（Windows: PowerShell / Unix: 任意のシェル）

**それ以外はすべて同じです。**

---

## 4. 初期セットアップ

### 設定：環境変数

Rosa の起動時に、環境変数 **`SESSION_KEY`** が必須です (`src/main.rs:703`)。

```rust
let session_key = std::env::var("SESSION_KEY").expect("SESSION_KEY is not set");
```

このキーは `.env` ファイルからも読み込まれます (`src/main.rs:701` の `dotenvy::dotenv()`)。

設定方法は2通りです。

#### 方法A: `.env` ファイルを使う（推奨）

プロジェクトルート（`Cargo.toml` と同じ場所）に `.env` というファイルを作り、以下の1行を記述します。

```dotenv
SESSION_KEY=あなたのセッションキー
```

#### 方法B: 環境変数を直接設定

**Linux / macOS:**

```bash
export SESSION_KEY="あなたのセッションキー"
cargo run --release
```

**Windows (PowerShell):**

```powershell
$env:SESSION_KEY = "あなたのセッションキー"
cargo run --release
```

### セッションキーの取得方法

`SESSION_KEY` は zinro.net のセッショントークンです。以下の手順で取得できます。

1. ブラウザで <https://zinro.net> にログインします
2. 開発者ツールを開きます（`F12` または `Cmd+Option+I`）
3. **Application** / **Storage** → **Cookies** に移動します
4. `session_key` という名前の Cookie を探します
5. その値をコピーします

これがあなたのセッションキーです。別の方法として、ネットワークリクエストの `Cookie` ヘッダを確認することもできます。Rosa はここで使用しています (`src/api.rs:194`)。

```rust
Cookie: session_key=...
```

> **セキュリティに関する注意：** `SESSION_KEY` はパスワードと同じです。他人に共有したり、公開リポジトリにコミットしたりしないでください。Gitを使う場合は `.env` を `.gitignore` に追加してください。

### 初回起動

1. Rosa を起動します（下の「Rosa を実行する」を参照）
2. 画面が TUI レイアウトに切り替わります
3. 初期メッセージ `RosaClient started` が表示されます (`src/main.rs:122`)
4. Rosa が 2秒ごとにサーバーをポーリング開始します
5. 接続成功時：右上のステータスが **● online** になります (`src/ui.rs:170`)
6. 接続失敗時：ステータスが **● offline** になり、チャット欄に赤いエラーが表示されます (`src/main.rs:229`)

---

## 5. 基本的な使い方

### Rosa を実行する

**方法A: `cargo run` を使う**

```bash
cargo run --release
```

**方法B: ビルド済みの実行ファイルを直接実行**

**Linux / macOS:**

```bash
./target/release/rosa
```

**Windows:**

```powershell
.\target\release\rosa.exe
```

> `SESSION_KEY` が設定されていないと `SESSION_KEY is not set` エラーが出ます (`src/main.rs:703`)。セットアップセクションで対応してください。

### 画面レイアウト

Rosa を起動すると、以下のようなレイアウトになります (`src/ui.rs:95`)。

```
┌ RosaClient  Explorer Chat Context Terminal          ● online ┐  ← ヘッダー
├──────────────┬─────────────────────────────────────────────┤
│ Rooms        │ Chat                                          │
│  ロビー       │  ● alice  こんにちは皆さん                       │
│  研究室      │  ● bob    やあ！                                │
│              │  ● ...                                        │
│ Users        │                                               │
│  alice       │                                               │
│  bob         │                                               │
├──────────────┤                                               │
│ Context      ├───────────────────────────────────────────────┤
│              │ Terminal   ❯ チャットメッセージをここに入力     │
│  online      │                                               │
│  ロビー      │                                               │
│  昼 3日目    │                                               │
│  3 / 10      │                                               │
├──────────────┴─────────────────────────────────────────────┤
│ NORMAL  Chat                              ロビー             │  ← ステータスバー
└─────────────────────────────────────────────────────────────┘
```

**各パネルの説明：**

- **Explorer**（左上）：部屋と参加者の一覧
- **Chat**（右上）：受信したメッセージ（送信者・内容）
- **Context**（左下）：接続状態・現在の部屋・ゲーム状況など
- **Terminal**（右下）：メッセージ入力欄
- **ステータスバー**（最下段）：現在のモード、アクティブなパネル、部屋名

### モード（Vim 風）

Rosa には4つのモードがあります。ステータスバーの左に現在のモードが表示されます。

| モード | 表示 | 役割 |
|--------|------|------|
| **Normal** | `NORMAL` | 移動・スクロール・各操作の起点。デフォルト。 |
| **Insert** | `INSERT` | メッセージを入力するモード。 |
| **Command** | `COMMAND` | コロンコマンド（`:q`、`:clear` など）を入力。 |
| **Search** | `SEARCH` | 検索クエリ（`/text`）を入力。 |

どのモードからでも **Esc** を押すと Normal モードに戻ります。

### 最初のメッセージを送ってみる

1. **`i`** を押して Insert モードに入ります
2. メッセージを入力します。例：
   ```
   こんにちは皆さん！
   ```
3. **`Enter`** を押して送信します
4. あなたのメッセージが「sending…」付きで表示されます
5. サーバーが確認したら、通常表示に変わります
6. **`Esc`** を押して Normal モードに戻ります

---

## 6. キーバインド・コマンド一覧

すべての操作は起動後の画面内で行います。**Rosa にはコマンドライン引数がありません。** 以下は Normal モードからのキー一覧です (`src/main.rs:367+`)。

### パネル移動

| キー | 動作 | 備考 |
|------|------|------|
| `Tab` | 次のパネル（Explorer → Chat → Context → Terminal → …） | `src/main.rs:503` |
| `Shift+Tab` | 前のパネル | `src/main.rs:512` |
| `Ctrl+w` `h` | 左へ | 2キーコンボ：Ctrl+w を押してから h |
| `Ctrl+w` `l` | 右へ | 2キーコンボ |
| `Ctrl+w` `j` | 下へ | 2キーコンボ |
| `Ctrl+w` `k` | 上へ | 2キーコンボ |

### カーソル移動・スクロール

| キー | 動作 |
|------|------|
| `j` または `↓` | 1行下 |
| `k` または `↑` | 1行上 |
| `Ctrl+d` | 8行下 |
| `Ctrl+u` | 8行上 |
| `Ctrl+f` | 16行下（ページ送り） |
| `Ctrl+b` | 16行上（ページ戻し） |
| `gg` | 先頭へジャンプ |
| `G` | 末尾へジャンプ |

Explorer パネルでは、`h` / `←` で **Rooms** に、`l` / `→` で **Users** に切り替えます (`src/main.rs:529`)。

### 入力・検索・モード変更

| キー | 動作 |
|------|------|
| `i`, `a`, `A`, `I`, `o`, `O` | Insert モードに入る（すべて同じ） |
| `:` | Command モードに入る |
| `/` | Search モードに入る |
| `n` | 次の検索結果へ |
| `N` | 前の検索結果へ |
| `Esc` | Normal モードに戻る（またはキャンセル） |
| `q` | アプリケーション終了 |

### メッセージ削除（Chat パネル）

| キー | 動作 |
|------|------|
| `x` | カーソル位置のメッセージを削除 |
| `dd` | カーソル位置のメッセージを削除 |

> **重要：** この削除は **手元の画面表示からのみ消すもの** です。サーバー上のメッセージは削除されません (`src/main.rs:318`)。個人的に非表示にするだけだと思ってください。

### Insert モード中のキー

| キー | 動作 |
|------|------|
| 文字キー | 入力欄に文字を追加 |
| `Backspace` | 1文字削除 |
| `Enter` | メッセージを送信 |
| `Esc` | Normal モードに戻る（下書きは破棄） |

### コロンコマンド（Command モード）

`:` を押してからコマンドを入力し、`Enter` で実行します (`src/main.rs:602`)。使用可能なコマンド：

| コマンド | 動作 | 使用例 |
|---------|------|--------|
| `:q` | Rosa を終了 | `:q` `Enter` |
| `:quit` | Rosa を終了（`:q` と同じ） | `:quit` `Enter` |
| `:clear` | チャット表示を全消去（サーバーには影響なし） | `:clear` `Enter` |

認識されないコマンドを入力すると Normal モードに戻ります (`src/main.rs:612`)。

### 検索（Search モード）

`/` を押してから検索語を入力し、`Enter` で実行します (`src/main.rs:631`)。

- **検索対象：** ユーザー名・メッセージ本文の両方
- **大文字小文字：** 区別しません (`src/main.rs:346`)
- **移動：** `n` で次、`N` で前の結果へ移動
- **結果：** Chat パネルに移動して、該当行にカーソルが移動します

使用例：

```
/こんにちは
```

---

## 7. よくある問題と対処方法

### エラー：「SESSION_KEY is not set」

**症状：** 起動直後に終了する。

**原因：** Rosa がセッションキーを見つけられない (`src/main.rs:703`)。

**対処：**

1. プロジェクトルートに `.env` ファイルを作り `SESSION_KEY=your_key` を記述する、または
2. ターミナルで環境変数を設定してから起動する（セットアップセクション「方法B」を参照）
3. ファイルとキー値が正しいか確認する

---

### ステータスが「● offline」で赤いエラーが表示される

**症状：** Rosa がサーバーに接続できない。

**原因：** ネットワーク不通、セッションキーが無効・期限切れ、またはサーバー側の問題。

**確認・対処：**

1. インターネット接続を確認する
2. ブラウザで <https://zinro.net> を開けるか試す
3. `SESSION_KEY` が最新か確認する（期限切れの可能性あり。zinro.net に再度ログインして取得）
4. チャット欄の赤いエラーメッセージを読む (`src/main.rs:229`)
5. zinro.net のサーバーが稼働中か確認する

---

### シンボルが「□」や「?」で表示される

**症状：** アイコンが文字化けして見える。

**原因：** ターミナルフォントが Nerd Fonts や記号に対応していない。

**対処：** Nerd Fonts 対応フォント（推奨：JetBrains Mono Nerd Font）に変更する。多くのターミナルなら設定から変更できます。

---

### 色がおかしい・見づらい

**症状：** 配色が変に見える・暗い。

**原因：** ターミナルが True Color (24ビット)非対応。

**対処：** True Color 対応のモダンターミナルを使う：

- **Linux:** `gnome-terminal`, `Konsole`, `Terminator`, `iTerm2`
- **macOS:** `iTerm2`、または標準 Terminal（設定で RGB Color を有効化）
- **Windows:** Windows Terminal（Windows 10 以降に内蔵）

---

### メッセージが反映されるまで時間がかかる

**症状：** 送信したのに画面に表示されるまで数秒待つ。

**原因：** Rosa は 2秒ごとにサーバーをポーリングするため、反映に時間がかかります (`src/api.rs:18`)。また API 呼び出しもレート制限されます (`src/api.rs:157`)。

**想定動作：** メッセージに「sending…」表示がされ、サーバー確認後に通常表示に変わります。通常は 2〜4秒です。

**対処：** これは仕様です。chat パネルの更新を待ちましょう。

---

### Rosa が終了できない・画面が戻らない

**症状：** 終了コマンドが効かない・ターミナル表示が壊れている。

**対処：**

1. Normal モードで **`q`** を押す、または
2. **`:q`** を入力して `Enter` を押す

Rosa は正常に終了してターミナルを復帰させます。表示が壊れた場合は、ターミナルで `reset` または `clear` コマンドを実行して復帰できます。

---

## 8. 開発者向け情報

### ポーリング間隔について

- サーバーへの問い合わせ間隔は **2000 ms（2秒）固定** です (`src/main.rs:711`)
- Context パネルには現在の設定値 (`poll_interval_ms`) が表示されます (`src/ui.rs:414`)
- **制限：** 現在はハードコードされています。設定ファイルや環境変数での変更には非対応です
- **カスタマイズ方法：** ソースを編集して再ビルドしてください

### 自動化・連携

Rosa は **対話的な TUI** として設計されており、非対話的な自動実行やパイプは用意されていません。

**メッセージの自動送信などを行いたい場合：**

- `src/api.rs` の `ApiClient`（`send_message` など）を直接使用する
- あなた自身で Rust プログラムを書き、これらの関数を呼び出す
- Rosa 本体には自動化用のインターフェースはありません

### API・ネットワーク詳細

| 項目 | 値 |
|------|-----|
| **サーバー** | `https://zinro.net`（`src/api.rs:16` で固定） |
| **メッセージ取得** | `GET /m/api/?mode=message` (`src/api.rs:239`) |
| **プレイヤー取得** | `GET /m/api/?mode=players` (`src/api.rs:262`) |
| **メッセージ送信** | `POST /m/player.php?mode=message&to_user=ALL&message=...` (`src/api.rs:205`, `src/api.rs:223`) |
| **認証** | Cookie ヘッダ：`session_key=<SESSION_KEY>` (`src/api.rs:194`) |

### Rust コマンドリファレンス

```bash
cargo build          # デバッグビルド
cargo build --release # リリース最適化ビルド
cargo run --release   # ビルド＆実行
cargo fmt             # コード整形
cargo clippy          # リント提案
```

---

## 貢献について

バグ報告やアイデアをお持ちですか？貢献をお待ちしています！

---

## License

MIT License

Copyright (c) 2026 negiradomoti

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## 質問がある場合

- GitHub でIssueを開く
- ソースコードを読む
- 開発者に聞いてみる

Rosa を楽しんでください！