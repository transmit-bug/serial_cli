import * as React from 'react'
import { Virtuoso } from 'react-virtuoso'
import { cn } from '@/lib/utils'

interface VirtualListProps<T> {
  data: T[]
  renderItem: (item: T, index: number) => React.ReactNode
  keyExtractor?: (item: T, index: number) => string
  height?: string | number
  className?: string
  overscan?: number
  itemHeight?: number | ((item: T, index: number) => number)
  emptyContent?: React.ReactNode
  loadingContent?: React.ReactNode
  defaultItemHeight?: number
}

export function VirtualList<T>({
  data,
  renderItem,
  keyExtractor,
  height = '100%',
  className,
  overscan = 100,
  itemHeight,
  emptyContent,
  loadingContent,
  defaultItemHeight = 40,
}: VirtualListProps<T>) {
  // Generate default key if not provided
  const getDefaultKey = (item: T, index: number) => {
    if (typeof item === 'object' && item !== null && 'id' in item) {
      return String((item as any).id)
    }
    return `${index}-${JSON.stringify(item)}`
  }

  const getKey = keyExtractor || getDefaultKey

  // Calculate item height
  const getItemHeight = React.useCallback(
    (item: T, index: number) => {
      if (typeof itemHeight === 'function') {
        return itemHeight(item, index)
      }
      if (typeof itemHeight === 'number') {
        return itemHeight
      }
      return defaultItemHeight
    },
    [itemHeight, defaultItemHeight]
  )

  // Render item
  const renderItemWrapper = React.useCallback(
    (index: number) => {
      const item = data[index]
      if (!item) return null

      return (
        <div
          data-index={index}
          style={{
            height: getItemHeight(item, index),
          }}
        >
          {renderItem(item, index)}
        </div>
      )
    },
    [data, renderItem, getItemHeight]
  )

  // Empty state
  if (data.length === 0 && emptyContent) {
    return (
      <div
        className={cn(
          'flex items-center justify-center h-full text-muted-foreground',
          className
        )}
        style={{ height }}
      >
        {emptyContent}
      </div>
    )
  }

  // Loading state
  if (loadingContent && data.length === 0) {
    return (
      <div
        className={cn(
          'flex items-center justify-center h-full text-muted-foreground',
          className
        )}
        style={{ height }}
      >
        {loadingContent}
      </div>
    )
  }

  return (
    <div
      className={cn('overflow-hidden', className)}
      style={{ height }}
    >
      <Virtuoso
        style={{ height: '100%' }}
        totalCount={data.length}
        itemContent={renderItemWrapper}
        overscan={overscan}
        defaultItemHeight={defaultItemHeight}
        components={{
          EmptyPlaceholder: () => (
            <div className="flex items-center justify-center h-full text-muted-foreground p-8">
              {emptyContent || 'No data'}
            </div>
          ),
        }}
      />
    </div>
  )
}

// Export a simpler version for common use cases
interface SimpleVirtualListProps<T> {
  data: T[]
  renderItem: (item: T) => React.ReactNode
  className?: string
  height?: string | number
  emptyMessage?: string
}

export function SimpleVirtualList<T>({
  data,
  renderItem,
  className,
  height = '100%',
  emptyMessage = 'No items',
}: SimpleVirtualListProps<T>) {
  return (
    <VirtualList
      data={data}
      renderItem={(item) => renderItem(item)}
      height={height}
      className={className}
      emptyContent={<div className="text-center text-muted-foreground p-4">{emptyMessage}</div>}
    />
  )
}
