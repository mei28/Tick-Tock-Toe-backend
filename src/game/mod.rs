// ゲームロジック関連のファイルをモジュールとしてまとめる
pub mod state; // すべての内容を公開
pub use state::GameState; // GameState 構造体をエクスポート
