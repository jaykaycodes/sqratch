import { Link } from '@tanstack/react-router'

import { cn } from '#/lib/utils'

import Icons from './icons'

export function DefaultNotFound() {
	return (
		<div className="flex h-screen w-screen flex-col items-center justify-center pb-12">
			<h1 className="mb-4 font-medium text-2xl">Page not found</h1>

			<Link className="btn" to="/">
				Go to Home
			</Link>
		</div>
	)
}

export function DefaultPending({ showText = true }: { showText?: boolean }) {
	return (
		<div className="flex h-full w-full flex-1 flex-col items-center justify-center gap-4 self-center">
			<Icons.Loader2 className="h-4 w-4 animate-spin" />
			<div className={cn('text-muted-foreground text-sm', !showText && 'sr-only')}>Loading...</div>
		</div>
	)
}
