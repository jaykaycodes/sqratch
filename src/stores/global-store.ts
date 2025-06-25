import { observable } from '@legendapp/state'

const GlobalStore$ = observable({
	devMode: false,
})

export default GlobalStore$
