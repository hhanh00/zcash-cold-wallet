# ZCash Cold Wallet

***By default this will work with TESTNET coins***

See the section below for mainnet coins

This is a paper wallet generator and a cold wallet for ZCash. You can create a new
sapling wallet or use your own if you have the secret key.

## Cold Wallet
You should have two computers. Only one of them is connected to the Internet.
The other one is air-gapped and never uses the network. All data exchanged between
the online and offline computer should be made through USB drive.

This program allows you to prepare a spending transaction from the online computer
which only has the viewing key. Then you transfer the transaction data to the offline
computer that has the secret key. On the offline computer, you sign the transaction.
This produces a complete transaction that you copy back to the first computer.
Finally, you submit the signed transaction to the network.

In this workflow, your secret key never leaves the offline computer.

# Requirements

Ideally, you would be running your own lightnode server and zcashd. However, if it
is not practical, you can point to a public lightnode server. Unfortunately, I do not
know if there is one out there. If you do, please contact me and I will update the doc.

The lightnode URL should be specified before the command. For example:

`zcash-coldwallet http://127.0.0.1:9067 sync`

Only `sync` and `submit` require the lightnode server.

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

## Initialize wallet and blocks databases

On the online computer, you need to run this command once. This creates the cache database
where we store the blocks we get from the lightnode server and the account database.

`zcash-coldwallet init-db`

## Initialize Account database

Then import your viewing key to the account database.

```
 zcash-coldwallet init-account zxviewtestsapling1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgccd0249gc86dqhzt0e5fm48p7luzfvx5e5fgpn7ecu33yk4pdjxp53xsyerjwmv7j4t64vvsxd6a0qzzqecpf93rp8n473hkh0rwrtrsmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nchzn4xk
```

## Sync

Connect to the lightnode server (by default on localhost) and grab the latest blocks.
It also scans the new blocks to update your account information. This command runs online.

`zcash-coldwallet sync`

```
Starting height: 1288000
Synced to 1289093
Scan completed
```

Note: We start from a checkpoint at block 1288000. If you have received notes 
before that time, they will not appear in your wallet. This checkpoint will
be updated from time to time.

## Get Balance

Get your current balance. If the result is not what you expect, check that you are
synced. This command works offline.

`zcash-coldwallet get-balance`

```
Balance: 100000000
```

## Prepare Spending Transaction

`zcash-coldwallet prepare-tx ztestsapling16vq8ue73sd3hqzjf4tlcgt2lkww4dl45te7rppwyafrjgxtgw9ac9vkxlyc0nehr4f5g58um63r 40000000`

Note: You may not be able to spend the entire balance you got from `get-balance` if
some received notes are too recent. If that's the case, wait for some blocks and 
resync.

The output will be a json object that you probably want to redirect to a file. This 
needs to be signed on the offline computer.

## Sign the transaction

Then you transfer the json file to the offline computer, for example in `tx.json`.
Now you need to sign with your secret key.

~~~
zcash-coldwallet sign secret-extended-key-test1qvf49wh8qqqqpqple3k2uqm97rkf24g5s5w5k40kdjyn7wnx2xrakjtfq9qw6wzjgemcuhmul4a34k2j0vlwk5rys9v57rr94zxu2j0f890qgsmfx5xq895m2glaphpftlar82w4duy2zcgaxtscy3hml9nxntxkzzhaccqysmexwxnhm3aydujfcva7ax8nxn9ckfjug3q92raw4vhp2f8q36g64muzl3e53d6zm897lq8gg3x7upjxwd7j7m4mtmhwjjx9pmyt0nc7u8mw6 `cat tx.json`
~~~

Note: This command will print out the payment address and the amount.
***Make sure these are correct!*** An attacker could trick
you into signing a transfer into his own account.

The output will be a string a hex numbers. That's the signed transaction that need to be broadcasted.

Save this output to a file and copy it back to the first computer.

## Broadcast the signed transaction

~~~
zcash-coldwallet submit `cat tx.raw`
~~~

The output should look like:

```
SendResponse { error_code: 0, error_message: "\"86a82d880bd3394390613aabc9919f2de9c1c1b28d95011ef4828bcd8f43b4bf\"" }
```

This means that everything went well and the tx id is `86a82d880bd3394390613aabc9919f2de9c1c1b28d95011ef4828bcd8f43b4bf`
See it on the testnet explorer: [here](https://explorer.testnet.z.cash/tx/86a82d880bd3394390613aabc9919f2de9c1c1b28d95011ef4828bcd8f43b4bf)

# Mainnet

If you want to use this tool for mainnet coins, this tool was not reviewed
or approved by the ECC. As such, you do it at your own risk. 
**I am not responsible for any potential loss of money.**

To use mainnet, you need to compile to tool passing the feature flag `mainnet`

~~~
cargo build --features mainnet
~~~


# TODO

- Handle reorgs



