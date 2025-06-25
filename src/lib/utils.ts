import { QueryClient } from '@tanstack/react-query'
import { type ClassValue, clsx } from 'clsx'
import { toast } from 'sonner'
import { twMerge } from 'tailwind-merge'

import { createTauRPCProxy } from './taurpc'

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs))
}

export function parseConnectionString(connectionString: string) {
	try {
		const url = new URL(connectionString)

		return {
			host: url.hostname,
			port: Number.parseInt(url.port || '5432', 10),
			database: url.pathname.slice(1),
			user: url.username,
			password: url.password,
		}
	} catch (error) {
		throw new Error(`Invalid connection string format: ${connectionString}\n${error}`)
	}
}

export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			retry: 1,
			retryDelay: (i) => Math.min(1000 * 2 ** i, 30_000),
			experimental_prefetchInRender: true,
		},
	},
})

export const taurpc = createTauRPCProxy()

export function copyToClipboard(text: string) {
	navigator.clipboard.writeText(text)
	toast.success('Copied to clipboard')
}
