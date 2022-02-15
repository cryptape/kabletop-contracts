# kabletop-contracts

Kabletop is a generic turn-based game framework built on CKB. Its network architecture is designed on a peer-to-peer basis, and is implemented by node-to-node interaction using state channel technology. A game developed on this framework will be able to implement both the game client logic and the CKB smart contract logic. Games will be developed in Lua and the framework itself is based on Rust and C.

Kabletop consists of four types of contracts: **nft contract**, **wallet contract**, **payment contract** and **Kabletop contract**. For more details refer to https://talk.nervos.org/t/kabletop/5715.

## NFT Contract

NFT Contract acts as a temporary simple NFT protocol for Kabletop soon to be replaced by other official NFT protocols such as Mibao. 

The NFT format proclaimed in the contract is just an NFT which represents 20 bytes, which can also be called H160. More NFTs you have in Kabletop means more CKBs to be staked. In Kabletop, By having more NFTs, users will need to pledge more CKBs for storing these NFTs. All staked CKBs can be seen as TVLs for Kabletop.

NFT can be also deleted and transferred to others, these two operations will release CKBs.

> Build contract:

``` sh
capsule build -n nft
```

> Run tests:

``` sh
cd tests/nft
cargo test -- --nocapture
```

## Wallet Contract

The wallet contract, or ownerlock contract, represents the wallet of the NFT creator. The wallet contract implements a similar logic to the ACP (AnyoneCanPay) contract. This contract requires use in conjunction with a payment contract.

> Build contract:

``` sh
capsule build -n wallet
```

> Run tests:

``` sh
cd tests/wallet
cargo test -- --nocapture
```

## Payment Contract

Payment contract listens and processes purchases from NFT owners. The payment contract should always run alongside the wallet contract.

If the owner of both the wallet and the payment contracts is also the creator of the NFT, then the data represents the incremental rules set by the creator for the NFT, i.e., the output probability of each NFT.

If the owners of the wallet contract and the payment contract are the creators of the NFT and the owner of the NFT respectively, the data represents the number of NFT card packs purchased by the owners so far.


> Build contract:

 ``` sh
 capsule build -n payment
 ```

> Run tests:

``` sh
cd tests/payment
cargo test -- --nocapture
```

## Kabletop Contract (or Channel Contract)

Kabletop contract is written in C and has fully integrated the Lua interpreter engine to run the Lua code that carries the GamePlay logic in CKB-VM.

The Kabletop contract only supports two-player turn-based matchmaking scenarios at the moment. Each player signs their opponent's turn data which will be placed in the Witnesses field eventually.


> Build contract:

 ``` sh
 capsule build -n kabletop
 ```

> Run tests:

``` sh
cd tests/kabletop
cargo test -- --nocapture
```
