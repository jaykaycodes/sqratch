import type React from 'react'

import { createLink, type LinkComponent } from '@tanstack/react-router'
import type { VariantProps } from 'class-variance-authority'

import { buttonVariants } from '#/components/ui/button'
import { cn } from '#/lib/utils'

interface AppLinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  disabled?: boolean
  /** @default 'link' */
  variant?: VariantProps<typeof buttonVariants>['variant']
  /** @default 'default' */
  size?: VariantProps<typeof buttonVariants>['size']
  ref?: React.Ref<HTMLAnchorElement>
}

const AppLinkComponent = ({
  className,
  children,
  disabled,
  variant = 'link',
  size = 'default',
  ref,
  ...props
}: AppLinkProps) => {
  return (
    <a
      ref={ref}
      className={cn(
        buttonVariants({ variant, size, className }),
        disabled && 'pointer-events-none opacity-50',
      )}
      {...props}
    >
      {children}
    </a>
  )
}

const CreatedAppLink = createLink(AppLinkComponent)

export const AppLink: LinkComponent<typeof AppLinkComponent> = (props) => (
  <CreatedAppLink preload="intent" {...props} />
)
