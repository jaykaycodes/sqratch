import React from 'react'

import * as CheckboxPrimitive from '@radix-ui/react-checkbox'
import { CheckIcon } from 'lucide-react'

import { cn } from '#/lib/utils'

function Checkbox({ className, ...props }: React.ComponentProps<typeof CheckboxPrimitive.Root>) {
	return (
		<CheckboxPrimitive.Root
			className={cn(
				'peer size-4 shrink-0 rounded-[4px] border border-input shadow-xs outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-4 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:focus-visible:ring-0 data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:outline-ring/40 dark:ring-ring/20',
				className,
			)}
			data-slot="checkbox"
			{...props}
		>
			<CheckboxPrimitive.Indicator
				className="flex items-center justify-center text-current"
				data-slot="checkbox-indicator"
			>
				<CheckIcon className="size-3.5" />
			</CheckboxPrimitive.Indicator>
		</CheckboxPrimitive.Root>
	)
}

export { Checkbox }
