import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { createTauRPCProxy } from './taurpc'

const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			staleTime: 30 * 1000,
			retry: 1,
			retryDelay: (i) => Math.min(1000 * 2 ** i, 30_000),
			experimental_prefetchInRender: true,
			throwOnError(error, query) {
				console.error(error)
				toast.error('Failed to fetch data', {
					id: query.queryKey.join('-'),
					description: error.message,
					duration: 10_000,
				})
				return false
			},
		},
	},
})

export default queryClient

export const taurpc = createTauRPCProxy()
