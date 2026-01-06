# SolidJS

## Tips and Tricks

### Reactivity not working

#### Collections as Signals

If the signal is for a collection and it is being modified internally, SolidJS will not notice the difference unless an entirely new collection is created on each update. To work-around this limitation, define a custom `equals` function, like so:

```javascript
const [selectedAssets, setSelectedAssets] = createSignal<Set<string>>(
  new Set(),
  {
    // avoid having to create a new set in order for SolidJS to notice
    equals: (prev, next) => prev.size !== next.size
  }
);
```

#### Resource not refetching

If the input signal to the resource is a collection that changes internally (like the `Set` example above), then SolidJS will not notice the change and thus not refetch the resource. Creating a new set each time seems to be the best approach.
