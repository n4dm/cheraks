# CHERAKS (CHK)

### A digital currency designed around open participation.

Cheraks is a proposed peer-to-peer digital currency inspired by Bitcoin. Its long-term vision is to let people transfer digital value directly between wallets through a shared blockchain, without relying on a single company to maintain the transaction ledger.

The currency is called **Cheraks**, and its planned ticker symbol is **CHK**.

Cheraks is being designed around three core ideas:

- **Limited supply:** a planned maximum of 21 million CHK.
- **Proof of work:** a proposed mining system for producing and validating blocks.
- **Open participation:** a future network where independent computers can verify transactions and maintain the blockchain.

Cheraks is currently experimental. These are the intended properties of the currency, not guarantees that a functioning public network already provides them.

---

## 1. What is Cheraks?

Cheraks is intended to be a digital currency that exists on a blockchain rather than in a bank account controlled by a single institution.

In a traditional payment system, a central provider typically maintains the authoritative record of balances and transfers. In a decentralized cryptocurrency, network participants independently verify transactions according to shared protocol rules.

The long-term goal for CHK is to provide a digital asset that users can hold in their own wallets and transfer to other users on the Cheraks network.

### The vision

- Send CHK between participating wallets.
- Receive CHK without needing an account at a central bank or payment company.
- Verify transactions using public blockchain data.
- Allow independent nodes to validate the network's rules.
- Use a publicly specified issuance policy with a planned maximum supply of 21 million CHK.

These capabilities require a complete blockchain, secure wallets, a functioning network, and extensive testing before they can be offered to users.

## 2. How a CHK payment is intended to work

Imagine Alice wants to send 10 CHK to Bob.

**Step 1 — Bob shares his receiving address.**

Bob creates or opens a Cheraks wallet and shares his public receiving address with Alice.

**Step 2 — Alice creates a transaction.**

Alice's wallet prepares a transaction specifying Bob's address and the amount to transfer. The wallet signs the transaction using Alice's private key.

**Step 3 — The network checks the transaction.**

Nodes would verify that the signature is valid, that Alice has enough spendable CHK, that the transaction follows the protocol rules, and that the same funds have not already been spent.

**Step 4 — A miner includes the transaction in a block.**

Under the intended proof-of-work design, a miner searches for a block that satisfies the network's difficulty target.

**Step 5 — Nodes verify the new block.**

Independent nodes check the block and its transactions before accepting it as part of their local chain.

**Step 6 — The payment gains confirmation.**

As additional valid blocks build on top of the block containing Alice's payment, the transaction gains more confirmations. Confirmation reduces the likelihood of a reorganization reversing the payment, but it does not provide an absolute guarantee.

```text
Alice's wallet
      |
      | Creates and signs a payment
      v
Network nodes validate transaction
      |
      v
Miner produces a valid proof of work
      |
      v
Nodes validate the block
      |
      v
Payment included in blockchain
      |
      v
Bob sees the confirmed payment
```

**Current limitation:** This describes the intended transaction flow. Cheraks does not yet have a complete transaction system, production wallet, or public network.

## 3. How CHK mining is intended to work

Cheraks is planned to use **proof of work (PoW)**, the consensus approach associated with Bitcoin.

Mining would involve specialized software repeatedly trying different candidate block values until a block hash satisfies the network's difficulty target.

The miner that discovers a valid block can broadcast it to the network. Other nodes independently check the proof and all the block's contents.

### Why proof of work?

Proof of work is intended to make it costly to produce competing blocks while allowing other participants to verify proposed blocks relatively cheaply.

The network would need clear rules for:

- What makes a block valid.
- How mining difficulty changes.
- How the winning chain is selected.
- What reward, if any, a valid block creates.
- How transactions and fees are handled.
- How invalid blocks and conflicting transactions are rejected.

Mining does not automatically make a coin valuable, guarantee fairness, or eliminate attacks. Security depends on the complete protocol and the resources supporting the network.

### Will users be able to mine CHK?

Mining is a planned feature, but public mining is not available yet. The current prototype demonstrates simplified proof of work locally; it is not connected to a live mining network.

## 4. CHK supply and monetary policy

Cheraks has a **planned maximum supply of 21,000,000 CHK**.

A maximum supply is intended to define an upper bound on the number of currency units that the protocol can create. To make that limit meaningful, every node must enforce the same issuance rules.

The final monetary policy still needs to specify the block reward, issuance schedule, reward reductions, smallest unit, and transaction-fee rules.

| Monetary property | Intended policy |
|---|---|
| Currency name | Cheraks |
| Ticker | CHK |
| Planned maximum supply | 21,000,000 CHK |
| Consensus approach | Proof of work |
| Block reward | Not finalized |
| Block interval | Not finalized |
| Reward reduction schedule | Not finalized |
| Smallest unit | To be specified |
| Public issuance | Not active |

The supply limit is a design target, not yet a production-enforced guarantee. No claim is made that 21 million CHK have been issued, mined, distributed, or made available to the public.

## 5. What gives CHK value?

A cryptocurrency does not acquire value simply because it has a name, a ticker, a limited supply, or mining software.

Any future market value for CHK would depend on factors such as:

- Whether the network works securely and reliably.
- Whether people find the currency useful.
- Whether users, developers, and businesses choose to participate.
- Whether the protocol and monetary rules are credible.
- Whether there is genuine demand and a functioning market.

Cheraks makes no promise of a price, return, listing, or future market demand. CHK should not be represented as an investment with guaranteed appreciation.

## 6. How Cheraks is intended to be decentralized

The long-term plan is for multiple independently operated nodes to maintain and verify the blockchain.

Rather than trusting one server's transaction history, each node would apply the protocol rules to the blocks and transactions it receives.

A decentralized network requires more than publishing source code. It needs independent operators, a working peer-to-peer protocol, reliable block synchronization, consistent validation rules, and protection against malicious or unavailable peers.

Cheraks has not yet launched this network. No public node endpoints or active network statistics are currently available.

## 7. Wallets, addresses, and private keys

In a mature Cheraks network, a wallet would help users manage their keys, generate receiving addresses, prepare transactions, and inspect payment status.

A receiving address is meant to be shareable. A private key must remain secret because it authorizes transactions for the funds it controls.

Users should never share private keys or recovery phrases with anyone claiming to offer CHK support.

A production wallet has not yet been released. Do not use the prototype to store real funds or treat test coins as valuable assets.

## 8. Transaction fees and confirmations

The final CHK protocol must define how transaction fees work and how users can estimate confirmation times.

In many proof-of-work networks, miners select transactions for blocks, and nodes independently validate them. Fees can influence which transactions miners choose, but they do not guarantee immediate inclusion.

Cheraks has not finalized its block interval, fee policy, or confirmation guidance. Therefore, no specific transaction speed, fee, or capacity should be promised at this stage.

## 9. Testnet and mainnet

A **testnet** is a separate network used to test software and protocol behavior. Testnet coins are intended to have no monetary value.

A **mainnet** is the production network where the protocol is used for real transactions.

Cheraks should launch a testnet only after it has a complete, verifiable transaction system, peer-to-peer networking, persistent storage, and consistent consensus rules. A mainnet should be considered only after substantial independent testing and security review.

| Network | Status |
|---|---|
| Local prototype | In early development |
| Public testnet | Not launched |
| Mainnet | Not launched |
| Production wallets | Not available |
| Official explorer | Not available |

No public CHK faucet, exchange listing, or official node service has been announced.

## 10. Frequently asked questions

**Is Cheraks a real, transferable cryptocurrency right now?**

No. Cheraks is an early software prototype with a proposed currency design. It does not yet have a functioning public blockchain on which users can transfer CHK.

**How many CHK exist today?**

There is no verified public supply. The planned maximum supply is 21 million CHK, but the production issuance rules have not been completed.

**Can I buy CHK?**

There is no official public sale or exchange listing announced by the project.

**Can I mine CHK today?**

The current prototype can demonstrate simplified local mining. It is not public mining and does not produce coins that can be transferred on a live Cheraks network.

**Can I use CHK to pay for products or services?**

Not currently through an official Cheraks network. That would require a working blockchain and willing recipients.

**Will CHK's price increase?**

There is no basis for promising a price or return. A future market may never develop.

**Why is the supply planned at 21 million?**

The initial design borrows the familiar 21-million-unit limit associated with Bitcoin. Cheraks must still finalize and enforce its own issuance schedule before the limit can be treated as a protocol guarantee.

**Is Cheraks affiliated with Bitcoin?**

No affiliation is implied. Cheraks is an independent project inspired by general proof-of-work cryptocurrency concepts.

## 11. Project status

Cheraks is being developed incrementally, with an emphasis on verifiable rules and transparent documentation.

The immediate objective is to turn the educational prototype into a rigorously specified blockchain, then build and test a multi-node testnet before considering any production release.

**Cheraks (CHK) — experimental digital currency, early development.**

*No public network, transferable currency, market price, or production security guarantee is currently claimed.*
