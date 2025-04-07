import React, { memo } from 'react'

import { Observable, observable } from '@legendapp/state'
import { use$, useMount, useObservable } from '@legendapp/state/react'

export function useElementIsScrolled(
	target: React.RefObject<HTMLElement> | HTMLElement,
	field: 'scrollTop' | 'scrollLeft' = 'scrollTop',
): Observable<boolean> {
	// Create an observable to store the isScrolled value
	const obs$ = useObservable(false)

	useMount(() => {
		const element: HTMLElement = target instanceof HTMLElement ? target : target.current

		// Handle element scroll and set it on the observable
		const handleScroll = () => {
			const isScrolled = element[field] > 0
			if (isScrolled !== obs$.peek()) {
				obs$.set(isScrolled)
			}
		}

		element.addEventListener('scroll', handleScroll)
		return () => {
			element.removeEventListener('scroll', handleScroll)
		}
	})

	return obs$
}

export function useElementScroll(
	target: React.RefObject<HTMLElement> | HTMLElement,
	field: 'scrollTop' | 'scrollLeft' = 'scrollTop',
): Observable<number> {
	// Create an observable to store the scroll value
	const obs$ = useObservable<number>(undefined)

	useMount(() => {
		const element: HTMLElement = target instanceof HTMLElement ? target : target.current

		// Handle element scroll and set it on the observable
		const handleScroll = () => {
			const scroll = element[field]
			if (scroll !== obs$.peek()) {
				obs$.set(scroll)
			}
		}

		element.addEventListener('scroll', handleScroll)
		return () => {
			element.removeEventListener('scroll', handleScroll)
		}
	})

	return obs$
}

// Get current focus state of the window
const hasFocus = () => typeof document !== 'undefined' && document.hasFocus()

// An observable with the current value of whether the window is focused
export const isWindowFocused$ = observable<boolean>(() => {
	// Setup the window focus event listeners, once when the observable is first accessed.
	const onFocus = () => {
		isWindowFocused$.set(true)
	}
	const onBlur = () => {
		isWindowFocused$.set(false)
	}
	window.addEventListener('focus', onFocus)
	window.addEventListener('blur', onBlur)

	// Return the current focused state
	return hasFocus()
})

// biome-ignore lint/suspicious/noExplicitAny: <explanation>
export type TypedMemo = <T extends React.ComponentType<any>>(
	Component: T,
	propsAreEqual?: (
		prevProps: Readonly<React.ComponentProps<T>>,
		nextProps: Readonly<React.ComponentProps<T>>,
	) => boolean,
) => T & { displayName?: string }

export const typedMemo = memo as TypedMemo

interface HookToObservableProps<T, T2> {
	value$: Observable<T>
	hook: () => T
	if?: (value: T) => boolean
	getValue?: (value: T) => T2
}

/**
 * HookToObservable connects React hooks to Observable state.
 *
 * This component takes the value from a React hook and sets it to an Observable.
 * It's useful for:
 * - Connecting React's built-in hooks (useWindowDimensions, useColorScheme, etc.) to Observable state
 * - Improving performance by not re-rendering whenever the hook updates
 * - Making hook values accessible to non-React code via Observables
 * - Conditionally updating Observable state based on hook values
 *
 * ### Example: Syncing window dimensions to Observable state
 *
 * ```tsx
 * import { memo } from 'react';
 * import { observable } from '@legendapp/state';
 * import { HookToObservable } from '@legendapp/state/react';
 * import { useWindowDimensions } from 'react-native';
 *
 * // Create an Observable to store window dimensions
 * const windowDimensions$ = observable({ width: 0, height: 0 });
 *
 * export const HookWindowDimensions = memo(function HookWindowDimensions() {
 *   return (
 *     <HookToObservable
 *       hook={useWindowDimensions}
 *       value$={windowDimensions$}
 *       if={() => !state$.showSettings.get()}
 *       getValue={(value) => ({ width: value.width, height: value.height })}
 *     />
 *   );
 * });
 * ```
 */
export const HookToObservable = typedMemo(function HookToObservable<T, T2 = T>({
	value$,
	hook,
	if: ifProp,
	getValue,
}: HookToObservableProps<T, T2>) {
	const value = hook()

	React.useLayoutEffect(() => {
		if (!ifProp || ifProp(value)) {
			const valueToSet = getValue ? getValue(value) : value
			// biome-ignore lint/suspicious/noExplicitAny: <explanation>
			;(value$ as Observable<any>).set(valueToSet)
		}
	}, [value, ifProp, value$, getValue])

	// This component doesn't render anything
	return null
})

type Selector<T> = Parameters<typeof use$<T>>[0]
type Options = Parameters<typeof use$<unknown>>[1]
export const useSuspended$ = <T>(selector: Selector<T>, options?: Options) =>
	use$<T>(selector, {
		...options,
		suspense: true,
	})
