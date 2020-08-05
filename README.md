# DBマイグレーションの使い方

## マイグレーションのテーブルを作成する
```sh
db-migraiton-tool --url postgres://user:pass@localhost:5432/test --init
```

## パッチを適用する(名前だけで判定する)
```sh
db-migraiton-tool --url postgres://user:pass@localhost:5432/test --dir db/sql/patches
```

## ストアードプロシージャを適用する(sha1で判定する)
```sh
db-migraiton-tool --url postgres://user:pass@localhost:5432/test --dir db/sql/stored --sha1
```

## ドライRun
```sh
db-migraiton-tool --url postgres://user:pass@localhost:5432/test --dir db/sql/stored --sha1 --dry_run
```