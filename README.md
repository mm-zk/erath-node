# Running L1 & L2 Together: Era + Reth


## Issues:
currently era requires a nightly rust compiler - and the latest one that works, doesn't support simd anymore - so small fixes had to be added to packed_simd and era-boojum.


Also the era_test_harness 1.4.0 was not upgraded with the latest boojum api change. So I had to 'move back' 1.4.1 and boojum to the PRs before the API change.




## Running:


cargo run -p erath-node -- node --http --enable-ext --dev --instance 1 --dev.block-time 1s --datadir /tmp/foo1


In the future - replace datadir with some tmpdir.



```shell
RUST_BACKTRACE=1 cargo run -p erath-node -- node --http --enable-ext --dev --instance 1 --dev.block-max-transactions 2 --datadir /tmp/foo1
```


Rich accounts (on L1):

m/44'/60'/0'/0/0	0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266	0x038318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75	0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

m/44'/60'/0'/0/1	0x70997970C51812dc3A010C7d01b50e0d17dc79C8	0x02ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0	0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
m/44'/60'/0'/0/2	0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC	0x039d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f04	0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
m/44'/60'/0'/0/3	0x90F79bf6EB2c4f870365E785982E1f101E93b906	0x0220b871f3ced029e14472ec4ebc3c0448164942b123aa6af91a3386c1c403e0eb	0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
m/44'/60'/0'/0/4	0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65	0x03bf6ee64a8d2fdc551ec8bb9ef862ef6b4bcb1805cdc520c3aa5866c0575fd3b5	0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a
m/44'/60'/0'/0/5	0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc	0x0337b84de6947b243626cc8b977bb1f1632610614842468dfa8f35dcbbc55a515e	0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba
m/44'/60'/0'/0/6	0x976EA74026E726554dB657fA54763abd0C3a0aa9	0x029a4ab212cb92775d227af4237c20b81f4221e9361d29007dfc16c79186b577cb	0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e
m/44'/60'/0'/0/7	0x14dC79964da2C08b23698B3D3cc7Ca32193d9955	0x0201f2bf1fa920e77a43c7aec2587d0b3814093420cc59a9b3ad66dd5734dda7be	0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356
m/44'/60'/0'/0/8	0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f	0x03931e7fda8da226f799f791eefc9afebcd7ae2b1b19a03c5eaa8d72122d9fe74d	0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97
m/44'/60'/0'/0/9	0xa0Ee7A142d267C1f36714E4a8F75612F20a79720	0x023255458e24278e31d5940f304b16300fdff3f6efd3e2a030b5818310ac67af45	0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6



cast balance -r http://localhost:8545 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266