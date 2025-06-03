import { Toaster as Sonner, ToasterProps } from 'sonner'

import { cn } from '#/lib/utils'

export default function Toaster(props: ToasterProps) {
	return (
		<Sonner
			className={cn('toast toast-end group/toast')}
			toastOptions={{
				classNames: {
					toast: 'group alert alert-soft',
					default: '',
					info: 'alert-info',
					error: 'alert-error',
					warning: 'alert-warning',
					success: 'alert-success',
					loading: 'alert-loading',
					description: 'text-2xs text-current/80',
					actionButton: 'btn btn-sm btn-primary',
					cancelButton: 'btn btn-sm btn-outline',
					closeButton: 'btn btn-sm btn-circle',
				},
			}}
			{...props}
		/>
	)
}
