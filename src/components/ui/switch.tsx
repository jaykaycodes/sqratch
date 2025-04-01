import React from 'react'

import * as SwitchPrimitive from '@radix-ui/react-switch'

import { cn } from '#/lib/utils'

function Switch({ className, ...props }: React.ComponentProps<typeof SwitchPrimitive.Root>) {
	return (
		<SwitchPrimitive.Root
			className={cn(
				'peer inline-flex h-5 w-9 shrink-0 items-center rounded-full border-2 border-transparent shadow-xs outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:outline-hidden focus-visible:ring-4 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:focus-visible:ring-0 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input dark:outline-ring/40 dark:ring-ring/20',
				className,
			)}
			data-slot="switch"
			{...props}
		>
			<SwitchPrimitive.Thumb
				className={cn(
					'pointer-events-none block size-4 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0',
				)}
				data-slot="switch-thumb"
			/>
		</SwitchPrimitive.Root>
	)
}

export { Switch }
