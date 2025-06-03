import { createQueryKeyStore } from "@lukemorales/query-key-factory";

import { taurpc } from "./utils";

const Q = createQueryKeyStore({
	db: {
		isConnected: {
			queryKey: null,
			queryFn: taurpc.db.is_connected,
		},
		entities: {
			queryKey: null,
			queryFn: taurpc.db.get_all_entities,
		},
	},
});

export default Q;
