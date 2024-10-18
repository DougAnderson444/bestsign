/**
 * Request a peer pin our plog data using PeerRequest
 * @param {Piper} piper - The Piper instance
 * @param {Uint8Array} request - The bytes
 * @param {string} peer_id - The peer_id of the peer we are connected to
 */
export async function peerRequest(piper, request, peer_id) {
	if (!peer_id) {
		console.error('peer_id is required to make a PeerRequest');
		return;
	}
	console.log('peerRequest:', request, peer_id);
	// Put in DHT (PutRecord) Key is VLAD, Value is CID
	let pr = {
		action: 'PeerRequest',
		request: Array.from(new Uint8Array(request)),
		peer_id
	};

	try {
		console.log('PeerRequest:', pr);
		let response = await piper.command(pr);
		console.log('PeerResponse:', response);
	} catch (e) {
		console.error(e);
	}
}
