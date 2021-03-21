# ZCash Cold Wallet

This is command-line utility implementing a cold wallet for ZCash.

For a detailed explanation on what a cold wallet is, please refer
to [Cold Storage](https://en.bitcoin.it/wiki/Cold_storage).

# Experimental Warning Notice

**The code has not been audited and should be used for experimental purposes only.**

Release Binaries are for TESTNET coins only.

## What does it do?

- ZCash Coldwallet can help you create and spend paper wallets
- **It helps you spend from any z-addr as long as you have the secret key**
- **It keeps your secret key usage from any online computer**. 
- It supports shielded (sapling) addresses

> This program splits the process of making a spending transaction in two.
Using two air-gapped computers, you prepare the transaction from the online computer
which only has the viewing key. Then you transfer the transaction data to the offline
computer that has the secret key. On the offline computer, you complete the transaction
by signing it. You copy the finalized transaction back to the first computer,
and you can submit it to the network.

In this workflow, your secret key never leaves the offline computer.

## What doesn't it do?

- It will not work well as a daily wallet. For this use case, mobile wallets are better suited.
- It does not support t-addr or sprout addresses

# Requirements

You need two computers.

One of them must be connected to the Internet. The other one
should be kept offline. Both computers do not have to run the same OS.

The tool is a command line utility. You must be able to use a command shell, i.e. be familiar
with navigating directories and executing commands.

Ideally, you would be running your own lightwalletd server and zcashd. But it is not a requirement
and by default the tool will use a public lightwalletd server.

# Installation

- Copy the binary on both the online and offline computers.
- Copy the zcash params from the online computer to the offline computer. They are in the `.zcash-params`
folder on Linux if you have installed `zcashd`. Or use this [download script](https://github.com/zcash/zcash/blob/master/zcutil/fetch-params.sh)

# Usage

**For maximum security, all data exchanged between the online and offline computer should be made via USB drive.**

## Private lightwallet URL

Pass the `-l` option to specify the URL of the lightwalletd

`zcash-coldwallet -l http://127.0.0.1:9067 sync`

Only `init-account`, `sync` and `submit` require the lightwalletd server.

Obviously, the offline computer does not need lightwalletd.

# Usage

## Generate new wallet

`zcash-coldwallet generate`

Example Output:

```
Seed Phrase: reform tower uncle cloud diamond sail region index buddy inside pool picture monitor mango snow salute find welcome drum trick vanish submit float source
Derivation Path: m/32'/1'/0'
Secret Key: secret-extended-key-test1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgemcuhmul4a34k2j0vlwk5rys9v57rr94zxu2j0f890qgsmfx5xq895m2glaphpftlar82w4duy2zcgaxtscy3hml9nxntxkzzhaccqysmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nc7u8mw6
Viewing Key: zxviewtestsapling1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgccd0249gc86dqhzt0e5fm48p7luzfvx5e5fgpn7ecu33yk4pdjxp53xsyerjwmv7j4t64vvsxd6a0qzzqecpf93rp8n473hkh0rwrtrsmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nchzn4xk
Address: ztestsapling16vq8ue73sd3hqzjf4tlcgt2lkww4dl45te7rppwyafrjgxtgw9ac9vkxlyc0nehr4f5g58um63r
```

Note: zcash-cold-wallet does not store this information. 

**YOU MUST KEEP IT SAFE!**

## Initialize wallet and blocks databases

On the online computer, you need to run this command once. This creates the cache database
where we store the blocks we get from the lightwalletd server and the account database.

`zcash-coldwallet init-db`

## Initialize Account database

Then import your viewing key to the account database.

If this is a new account:

```
 zcash-coldwallet init-account zxviewtestsapling1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgccd0249gc86dqhzt0e5fm48p7luzfvx5e5fgpn7ecu33yk4pdjxp53xsyerjwmv7j4t64vvsxd6a0qzzqecpf93rp8n473hkh0rwrtrsmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nchzn4xk
```

If this is an old account

```
 zcash-coldwallet init-account zxviewtestsapling1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgccd0249gc86dqhzt0e5fm48p7luzfvx5e5fgpn7ecu33yk4pdjxp53xsyerjwmv7j4t64vvsxd6a0qzzqecpf93rp8n473hkh0rwrtrsmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nchzn4xk 2020-05-02
```
 
Notice the date that was passed as last argument. It is the account "birthday".

When the wallet syncs an old account, it needs to download past blocks and scan
them for your transactions. Pass the "birthday" of your account in order
to avoid getting earlier blocks. For example, if you know your wallet was created
around Mid Feb 2021, you can use 2021-02-01. It does not have to be precise, in
fact, a few days of margin does not harm.

## Sync

Connect to the lightwalletd server (by default ligthwalletd.com) and grab the latest blocks.
It also scans the new blocks to update your account information. This command runs online.

`zcash-coldwallet sync`

```
Starting height: 1288000
Synced to 1289093
Scan completed
```

**This command can take a few minutes depending on the starting
height**.

Note: We start from a checkpoint at the wallet birthday. If you have received notes
before that time, they will not appear in your wallet. 

Tip: If you are planning to sync many blocks, using
a RAM drive speeds up the process greatly. On Linux machines,
`/tmp` acts like a RAM drive. *You may get 100x speed* even over a SSD. This is highly recommended.

## Get Balance

Get your current balance. If the result is not what you expect, check that you are
synced. *This command works offline*.

`zcash-coldwallet get-balance`

```
Balance: 1.0
```

## Prepare Spending Transaction

`zcash-coldwallet prepare-tx ztestsapling16vq8ue73sd3hqzjf4tlcgt2lkww4dl45te7rppwyafrjgxtgw9ac9vkxlyc0nehr4f5g58um63r 0.4 tx.json`

The output will be a json file `tx.json`. This
needs to be signed on the offline computer.

## Sign the transaction

Transfer the json file to the offline computer by using a USB key for example (do not use the network).
Now you need to sign with your secret key.

~~~
zcash-coldwallet sign secret-extended-key-test1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgemcuhmul4a34k2j0vlwk5rys9v57rr94zxu2j0f890qgsmfx5xq895m2glaphpftlar82w4duy2zcgaxtscy3hml9nxntxkzzhaccqysmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nc7u8mw6 tx.json tx.raw
~~~

Note: This command will print out the payment address and the amount.
***Make sure these are correct!*** An attacker could trick
you into signing a transfer into his own account.

The output will be a signed transaction file `tx.raw`.

Transfer this file back to the first computer.

## Broadcast the signed transaction

~~~
zcash-coldwallet submit tx.raw
~~~

The output should look like:

```
Success! tx id: "86a82d880bd3394390613aabc9919f2de9c1c1b28d95011ef4828bcd8f43b4bf"
```

This means that everything went well and the tx id is `86a82d880bd3394390613aabc9919f2de9c1c1b28d95011ef4828bcd8f43b4bf`
See it on the testnet explorer: [here](https://explorer.testnet.z.cash/tx/86a82d880bd3394390613aabc9919f2de9c1c1b28d95011ef4828bcd8f43b4bf)

# Mainnet

If you want to use this tool for mainnet coins, this tool was not yet audited
by the ZOMG. As such, you do it at your own risk.
**I am not responsible for any potential loss of money.**

To use mainnet, you need to compile with the feature flag `mainnet`

~~~
cargo build --features mainnet --release
~~~

# FAQ

- Why not use a paper wallet?
  
  With a paper wallet, you can receive ZEC but not spend any. Without this tool, you would have to
  import your secret key to an online wallet when you want to send from your paper wallet,
  therefore reducing its safety.
  
- **I received some coins, and I just synced. I don't see my balance updated. Where are my coins?**

  Coins need *10 blocks* to mature before they are spendable. Before then, they will not appear in your
balance. 
  
- What happens if there is a blockchain reorg?

  The sync stops 10 blocks short of the latest block. If the re-org is shorter than 10 blocks (which
  should be the case unless the network has an issue), you shouldn't be affected. If there is 
  a longer reorg, you will need to manually delete the database files (*.sqlite3). I don't think
  it has ever happened though.
  
- How about using a hardware wallet?

  At this point, hardware wallets do not support z-addr. In any case, a hardware wallet is special
equipment that not everyone is comfortable using. ZCash Cold Wallet code is open source and
  does not rely on any proprietary logic. Also, hardware wallets break, but a paper wallet
  does not.
  
- Is there a UI?

  To keep the tool as simple as possible, the tool is command line only.

- Can I import more than one z-addr?

  No, it supports a single address. But, all the data is stored in a couple of files, you can keep
several sets of files.
  
- Does the tool store my secret key?

  **It does not store your secret anywhere**. Your viewing key is stored in the online computer, but 
your secret key is not kept in the online computer or the offline computer. The tool uses two SQLite
  database files. If you want to wipe out everything, just delete these two files.

- Does this tool work with mainnet coins?

  The release binaries are for TESTNET coins. 
  At this point, if you want to use mainnet coins, you must build it yourself. Once the source code
  is audited and approved by the ZOMG, mainnet binaries will be published as well.
  
  **By default this will work with TESTNET coins.** 
  
  See the section below for mainnet coins
  
- (Advanced usage) Can I swap the account file (data.sqlite3)?

  Yes but only if your new account file has a birthday after the original one. If you need
  to reindex, do the following:
    - delete/move the old data.sqlite3 file,
    - run `init-db`
    - import a new account with `init-account`
    - run `re-index`

  Your new balance should show up. If you want to be sure, just delete both data files and 
  start anew. 
    