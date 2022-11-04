# Zatlin-rs
## 概要
これは[Zatlin](https://github.com/Ziphil/Zatlin)のRust移植版のライブラリです．

## 本家と異なる部分
把握している限りの相違部分は以下のものとなります．
* 式で作成された結果に除外文字列が含まれていた場合のリトライ回数の指定が可能．

## zatlinマクロ
zatlinマクロを使用することで，Rustでzatlin構文をほぼそのままで記述することが可能となります．
```rust
let data = zatlin! {
    Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m";
    Ce = "" | "b" | "d" | "g" | "m" | "n" | "h";
    
    Va = "a" | "á" | "à" | "ä";
    Ve = "e" | "é" | "è" | "ë";
    Vi = "i" | "í" | "ì" | "ï";
    Vo = "o" | "ó" | "ò" | "ö";
    Vu = "u" | "ú" | "ù" | "ü";
    Vy = "y" | "ý" | "ỳ" | "ÿ";
    
    Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
    Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i";
    Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu;
    
    % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ "y" | ^ "ý" | ^ "ỳ" | ^ "ÿ" | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
}?;
let result = zatlin::generate_by(&data)?;
```
マクロを使用する場合には，featuresにて"use_macro"を指定する必要があります．
```toml
[dependencies]
zatlin = { version = "0.2", features = [ "use_macro" ] }
```

### マクロの制限
* 行末でのセミコロンの省略ができない．

## Unofficialの機能
* 丸かっこの使用が可能
* 変数の中のパターンを一段階処理した状態でローカル変数に束縛することが可能