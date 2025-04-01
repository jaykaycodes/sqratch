import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

const createQueryClient = () =>
	new QueryClient({
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

let clientQueryClientSingleton: QueryClient | undefined = undefined

export const getQueryClient = () => {
	if (typeof window === 'undefined') {
		// Server: always make a new query client
		return createQueryClient()
	}
	// Browser: use singleton pattern to keep the same query client
	clientQueryClientSingleton ??= createQueryClient()

	return clientQueryClientSingleton
}
