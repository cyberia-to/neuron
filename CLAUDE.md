# cell — working instructions

The user's requested canonical specification directory is specs/.
docs/ explains the model; specs/ defines it. Root README states implementation status.

The user established cyb/anatomy.md as the authority for organ meanings.
Cell unifies the runtime organ, local hosting and the protocol cell ladder.
The cyb log organ presents history. Cybergraph owns the history/write interface;
bbg and its storage backends own durable representation and storage mechanics.
Cell coordinates transitions through that interface.

The user approved starting implementation from the 0.2 design on 2026-09-11.
Update contracts before implementing behavior. Track implemented and unsupported
capabilities explicitly; implementation authorization is not a conformance claim.

Keep hashing in hemera, commitments/storage in bbg/lens, language/evaluation in
rune/nox, ordering/finality in foculus, transport in radio, framing in tape,
authorization in ward, secret operations in vault and presentation in prysm.
Use cyber's canonical particle, cyberlink and neuron vocabulary.
Runtime-free model code must remain usable without Bevy.

Use Nushell for scripting, rg for source discovery, and atomic conventional
commits. Push only when requested. Verify Markdown links, cross-file contracts
and declared implementation status for specification changes.
