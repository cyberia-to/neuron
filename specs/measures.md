---
tags: cyber, mudra, reference
crystal-type: reference
crystal-domain: cyber
alias: neuron measures, neuron metrics, how to measure a neuron, neuron scalars, neuron-measures
---
# neuron measures

every scalar the protocol attaches to a [[neuron]], in one place. a neuron is a [[particle]] (its native id is `Hemera(compressed public key)` — [[neuron/specs/identity|identity]] — and hashes are particles), so it carries every per-particle measure plus the per-neuron ones. this page names them, says what each measures, and points at the repo that owns the computation.

## the measures

| measure | scope | what it is | type | transferable | owner |
|---|---|---|---|---|---|
| [[identity]] | neuron | Hemera hash of the neuron secret; the address, itself a [[particle]] | $\mathbb{F}_p^4$ | no | mudra |
| [[cyberank]] | particle | collective [[focus]] that lands on a node; the node's $\phi^*$ coordinate | $[0,1]$ | no | tru |
| [[karma]] | neuron | accumulated [[Bayesian Truth Serum]] score; epistemic track record | $\mathbb{R}$ | no | tru |
| focus balance | neuron | spendable energy; consumed per [[cyberlink]] (the [[costly signal]]) | $\mathbb{R}_+$ | yes | tok |
| stake | neuron | $CYB locked across the neuron's conviction boxes | $\mathbb{R}_+$ | locked | tok |
| [[signals]] | neuron | ordered history of committed [[signals]] | list | — | cybergraph |
| conviction | cyberlink | $CYB committed to one edge (the amount $a$) | $\mathbb{R}_+$ | locked | tok |
| [[valence]] | cyberlink | the BTS meta-prediction $v \in \{-1, 0, +1\}$ | ternary | — | cybergraph |
| syntropy share | neuron | net information the neuron added: $\sum \Delta J$ over its [[signals]] | $\mathbb{R}$ | no | tru |

## two reputations, not one

a neuron carries two distinct reputational scalars, and they answer different questions:

- [[cyberank]] — reputation as attention. because the neuron's [[identity]] is a [[particle]], the [[tri-kernel]] assigns it a $\phi^*$ coordinate like any node: how much collective [[focus]] flows to the neuron itself.
- [[karma]] — reputation as track record. the accumulated [[Bayesian Truth Serum]] score: how often the neuron's [[cyberlinks]] carried genuine private signal the crowd had not yet priced in.

cyberank can be earned by being linked-to. karma can only be earned by being right before the crowd, and cannot be bought — it is the one measure that capital cannot purchase.

## the composite

three of these combine into the weight a neuron's [[cyberlink]] carries in the [[tri-kernel]] effective adjacency:

$$A^{\text{eff}}_{pq} = \sum_\ell \underbrace{\text{stake}(\ell)}_{\text{capital}} \times \underbrace{\text{karma}(\nu(\ell))}_{\text{track record}} \times \underbrace{f(\text{ICBS price}(\ell))}_{\text{market belief}}$$

capital and market belief are buyable; karma is not. this is what keeps the value function honest before [[Shapley]] attribution splits it.

## the risk dial

[[valence]] is a per-[[cyberlink]] choice of risk exposure, not a fixed tax:

- $v = 0$ — passive stake. the conviction still weights the edge in $A^{\text{eff}}$, so it affects the [[cyberank|rank]], but it earns no staking reward. structure for free, yield for none.
- $v = \pm 1$ — active epistemic bet. the conviction is wagered through the BTS zero-sum and earns from it: right pays, wrong pays the right.

a neuron picks its exposure link by link. passive capital shapes the graph but extracts no rent — reward flows only to non-zero [[valence]]. this is what keeps wealth from compounding by sitting still: idle stake cannot earn, only correct risk can.

## reward, in terms of these

the [[impulse]] reward a neuron mints is its [[Shapley]] share of the joint focus shift, computed on the [[karma]]-weighted effective graph and bounded by the real global $\Delta\phi^*$:

$$\text{mint}(\nu) = \text{Shapley}_\nu(v), \qquad v(S) = \Delta\phi^*\big(\text{effective graph} + S\big)$$

[[karma]] enters through the value function $v$ (so copies and noise carry near-zero marginal); [[Shapley]] splits the result fairly and conserves by its efficiency axiom. because $v(S)$ is weighted by stake, splitting one neuron into many Sybils with the same total stake yields the same total share — stake-weighting makes the attribution split-proof. identity is cheap; stake and [[karma]] are not. see [[cyber/rewards]] for the full specification.

## see also

- [[identity]] — the cryptographic self (mudra owns this; the rest are referenced)
- [[neuron]] — the agent these measures describe
- [[karma]], [[cyberank]], [[valence]], [[focus]], [[Shapley]]
- [[cyber/rewards]] — how these measures become reward
