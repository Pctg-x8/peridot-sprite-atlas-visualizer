## Rust + Windows App SDK 覚書

### NuGet を使う

Microsoft.WindowsAppSDK を入れるのに必要

NuGet は NuGet CLI で使う
vcxproj みたいなものはないので packages.config を使う方法でパッケージを管理する

既定では変なディレクトリにパッケージが配置されるので、プロジェクトから見えやすいところに配置するようにする。

```sh
$ nuget restore ./packages.config -PackagesDirectory ./.nuget
```

### Windows App SDK の初期化/後始末

`Microsoft.WindowsAppRuntime.Bootstrap.dll`の`MddBootstrapInitialize2`で初期化する
終了時には同 DLL の`MddBootstrapShutdown`を呼ぶ

DLL は exe から見える位置に何らかの方法で配置する（このプロジェクトでは`build.rs`でコピーすることにした）

`MddBootstrapInitialize2`に指定するバージョン番号は正確に一致しないといけない（例えば Runtime 1.6 がインストールされていたとして、1.0 を指定しても 1.6 を使ってくれるようになるわけではない）ので注意

### windows-rs 0.60

0.60 からなんかクレートの構成がいろいろ変わったらしい

- `Vector2`や`Vector3`は`windows-numerics`に分離（`Foundation.Numerics`は不要になった）
- 一部の共通コレクション型は`windows-collections`に分離（`Foundation.Collections`は不要になった）
- これらの変更に伴い、一部の型については bindgen での reference の参照指定が不要になった
