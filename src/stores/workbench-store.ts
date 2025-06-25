import { observable } from '@legendapp/state'

import { WORKBENCH_TABS, type WorkbenchTab } from '#/routes/-project/workbench'

const WorkbenchStore$ = observable({
	open: true,
	activeTab: WORKBENCH_TABS[0].label as WorkbenchTab,
})

export default WorkbenchStore$
