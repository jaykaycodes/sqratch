import { useCallback, useRef } from 'react'

interface UseDoubleClickOptions {
	/**
	 * Delay in milliseconds to wait before executing single click action.
	 * This allows time to detect if a double click is occurring.
	 * @default 300
	 */
	delay?: number
}

/**
 * Hook that handles both single and double click actions.
 * Single click action is delayed to allow detection of double clicks.
 * 
 * @param onSingleClick - Function to call on single click
 * @param onDoubleClick - Function to call on double click
 * @param options - Configuration options
 * @returns Click handler function to attach to element
 */
export function useDoubleClick<T = unknown>(
	onSingleClick: (event: T) => void,
	onDoubleClick: (event: T) => void,
	options: UseDoubleClickOptions = {}
) {
	const { delay = 300 } = options
	const clickTimeoutRef = useRef<NodeJS.Timeout | null>(null)
	const clickCountRef = useRef(0)
	const lastEventRef = useRef<T | null>(null)

	const handleClick = useCallback(
		(event: T) => {
			lastEventRef.current = event
			clickCountRef.current += 1

			if (clickCountRef.current === 1) {
				// First click - set timeout for single click
				clickTimeoutRef.current = setTimeout(() => {
					if (clickCountRef.current === 1 && lastEventRef.current) {
						onSingleClick(lastEventRef.current)
					}
					clickCountRef.current = 0
					lastEventRef.current = null
				}, delay)
			} else if (clickCountRef.current === 2) {
				// Second click - clear timeout and execute double click
				if (clickTimeoutRef.current) {
					clearTimeout(clickTimeoutRef.current)
					clickTimeoutRef.current = null
				}
				onDoubleClick(event)
				clickCountRef.current = 0
				lastEventRef.current = null
			}
		},
		[onSingleClick, onDoubleClick, delay]
	)

	// Cleanup timeout on unmount
	const cleanup = useCallback(() => {
		if (clickTimeoutRef.current) {
			clearTimeout(clickTimeoutRef.current)
			clickTimeoutRef.current = null
		}
		clickCountRef.current = 0
		lastEventRef.current = null
	}, [])

	// Return both the handler and cleanup function
	return { handleClick, cleanup }
}
