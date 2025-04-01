import React from 'react'

import { Slot } from '@radix-ui/react-slot'
import { createFormHook, createFormHookContexts, useStore } from '@tanstack/react-form'

import { Label } from '#/components/ui/label'
import { cn } from '#/lib/utils'

const {
	fieldContext,
	formContext,
	useFieldContext: _useFieldContext,
	useFormContext,
} = createFormHookContexts()

const { useAppForm, withForm } = createFormHook({
	fieldContext,
	formContext,
	fieldComponents: {
		FormLabel,
		FormControl,
		FormDescription,
		FormMessage,
		FormItem,
	},
	formComponents: {},
})

type FormItemContextValue = {
	id: string
}

const FormItemContext = React.createContext<FormItemContextValue>({} as FormItemContextValue)

function FormItem({ className, ...props }: React.ComponentProps<'div'>) {
	const id = React.useId()

	return (
		<FormItemContext.Provider value={{ id }}>
			<div className={cn('grid gap-2', className)} data-slot="form-item" {...props} />
		</FormItemContext.Provider>
	)
}

const useFieldContext = () => {
	const { id } = React.useContext(FormItemContext)
	const { name, store, ...fieldContext } = _useFieldContext()

	const errors = useStore(store, (state) => state.meta.errors)
	if (!fieldContext) {
		throw new Error('useFieldContext should be used within <FormItem>')
	}

	return {
		id,
		name,
		formItemId: `${id}-form-item`,
		formDescriptionId: `${id}-form-item-description`,
		formMessageId: `${id}-form-item-message`,
		errors,
		store,
		...fieldContext,
	}
}

function FormLabel({ className, ...props }: React.ComponentProps<typeof Label>) {
	const { formItemId, errors } = useFieldContext()

	return (
		<Label
			className={cn('data-[error=true]:text-destructive', className)}
			data-error={!!errors.length}
			data-slot="form-label"
			htmlFor={formItemId}
			{...props}
		/>
	)
}

function FormControl({ ...props }: React.ComponentProps<typeof Slot>) {
	const { errors, formItemId, formDescriptionId, formMessageId } = useFieldContext()

	return (
		<Slot
			aria-describedby={
				!errors.length ? `${formDescriptionId}` : `${formDescriptionId} ${formMessageId}`
			}
			aria-invalid={!!errors.length}
			data-slot="form-control"
			id={formItemId}
			{...props}
		/>
	)
}

function FormDescription({ className, ...props }: React.ComponentProps<'p'>) {
	const { formDescriptionId } = useFieldContext()

	return (
		<p
			className={cn('text-muted-foreground text-sm', className)}
			data-slot="form-description"
			id={formDescriptionId}
			{...props}
		/>
	)
}

function FormMessage({ className, ...props }: React.ComponentProps<'p'>) {
	const { errors, formMessageId } = useFieldContext()
	const body = errors.length ? String(errors.at(0)?.message ?? '') : props.children
	if (!body) return null

	return (
		<p
			className={cn('text-destructive text-sm', className)}
			data-slot="form-message"
			id={formMessageId}
			{...props}
		>
			{body}
		</p>
	)
}

export { useAppForm, useFormContext, useFieldContext, withForm }
