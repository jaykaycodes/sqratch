export default function DivButton({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			onKeyUp={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault()
					props.onClick?.(e as any)
				}
			}}
			role="button"
			tabIndex={0}
			{...props}
			className={className}
		/>
	)
}
