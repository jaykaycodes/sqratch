import type React from 'react'

import { createLink } from '@tanstack/react-router'

import { type ButtonVariantProps, buttonVariants } from '#/components/ui/button'
import { cn } from '#/lib/utils'

const AppLinkComponent = ({
	className,
	variant = 'link',
	size = 'default',
	...props
}: React.ComponentProps<'a'> & ButtonVariantProps) => (
	<a className={cn(buttonVariants({ variant, size, className }))} {...props} />
)
export const AppLink = createLink(AppLinkComponent)
