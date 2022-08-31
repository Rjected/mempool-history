# mempool-history

This tool uses ethereum pending transaction RPCs to listen for mempool transactions and store them.
It will also log **when** we are notified about a transaction (or transaction hash), allowing us to order mempool transactions by time of receipt.

This does NOT:
 * Track when the _provider or node_ received the transaction over p2p.
 * Track when the transaction was _sent_ over p2p.
 * Capture every single transaction-related p2p message.

The purpose of this tool is to gather data so it can be easily replayed while testing other applications.
