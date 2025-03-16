import React from 'react'

import { SelectValue } from '@radix-ui/react-select'
import { capitalCase } from 'change-case'
import {
  addDays,
  format,
  isDate,
  startOfMonth,
  startOfWeek,
  startOfYear,
  subDays,
  subYears,
} from 'date-fns'
import * as Icons from 'lucide-react'
import type { DateRange } from 'react-day-picker'

import { Button } from '#/components/ui/button'
import { Calendar, type CalendarProps } from '#/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '#/components/ui/popover'
import { cn } from '#/lib/utils'

import { Select, SelectContent, SelectItem, SelectTrigger } from './ui/select'

interface DatePickerProps<T extends Date | DateRange> {
  value?: T
  onChange?: (date: T) => void
  presets?: T extends Date ? (keyof typeof DatePresets)[] : (keyof typeof RangePresets)[]
  buttonClassName?: string
  popoverClassName?: string
  calendarProps?: Omit<CalendarProps, 'mode' | 'selected' | 'onSelect'>
}

export function DatePicker<T extends Date | DateRange>({
  value: _v,
  onChange,
  presets,
  buttonClassName,
  popoverClassName,
  calendarProps,
}: DatePickerProps<T>) {
  const [value, setValue] = React.useState<T | undefined>(_v)
  React.useEffect(() => setValue(_v), [_v])

  const handleValueChange = (selectedDate: T | undefined) => {
    setValue(selectedDate)
    if (selectedDate) onChange?.(selectedDate)
  }

  // keeps TS happy
  const renderCalendar = () =>
    isDate(value) ? (
      <Calendar
        defaultMonth={value}
        {...calendarProps}
        mode="single"
        selected={value}
        onSelect={handleValueChange as (date: Date | undefined) => void}
      />
    ) : (
      <Calendar
        defaultMonth={value?.from}
        {...calendarProps}
        mode="range"
        selected={value}
        onSelect={handleValueChange as (date: DateRange | undefined) => void}
      />
    )

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant={'outline'}
          className={cn(
            'justify-start text-left font-normal',
            !value && 'text-muted-foreground',
            buttonClassName,
          )}
        >
          <Icons.Calendar className="mr-2 h-4 w-4" />
          {!isDate(value) && value?.from ? (
            value.to ? (
              `${format(value.from, 'LLL dd, y')} - ${format(value.to, 'LLL dd, y')}`
            ) : (
              format(value.from, 'LLL dd, y')
            )
          ) : (
            <span>Pick a date</span>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className={cn(
          'w-auto',
          presets && presets.length > 0 ? 'flex flex-col gap-2 p-2' : 'p-0',
          popoverClassName,
        )}
      >
        {presets && presets.length > 0 ? (
          <>
            <Select
              onValueChange={(newValue) => {
                const fn = isDate(value)
                  ? DatePresets[newValue as keyof typeof DatePresets]
                  : RangePresets[newValue as keyof typeof RangePresets]
                handleValueChange(fn() as T)
              }}
            >
              <SelectTrigger>
                <SelectValue placeholder="Presets" />
              </SelectTrigger>
              <SelectContent position="popper">
                {presets.map((preset) => (
                  <SelectItem key={preset} value={preset}>
                    {capitalCase(preset)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <div className="rounded-md border">{renderCalendar()}</div>
          </>
        ) : (
          renderCalendar()
        )}
      </PopoverContent>
    </Popover>
  )
}

const DatePresets = {
  yesterday: () => subDays(new Date(), 1),
  today: () => new Date(),
  tomorrow: () => addDays(new Date(), 1),
} as const satisfies Record<string, () => Date>

const RangePresets = {
  this_week: () => ({
    from: startOfWeek(new Date(), { weekStartsOn: 1 }),
    to: new Date(),
  }),

  this_month: () => ({
    from: startOfMonth(new Date()),
    to: new Date(),
  }),

  this_year: () => ({
    from: startOfYear(new Date()),
    to: new Date(),
  }),

  previous_week: () => ({
    from: subDays(startOfWeek(new Date(), { weekStartsOn: 1 }), 7),
    to: new Date(),
  }),

  previous_month: () => ({
    from: subDays(startOfMonth(new Date()), 30),
    to: new Date(),
  }),

  previous_year: () => ({
    from: subYears(startOfYear(new Date()), 1),
    to: new Date(),
  }),

  last_7_days: () => ({
    from: subDays(new Date(), 7),
    to: new Date(),
  }),
  last_30_days: () => ({
    from: subDays(new Date(), 30),
    to: new Date(),
  }),
  last_90_days: () => ({
    from: subDays(new Date(), 90),
    to: new Date(),
  }),
} as const satisfies Record<string, () => DateRange>
