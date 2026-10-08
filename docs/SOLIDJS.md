# SolidJS

## Tips and Tricks

### Collections as Signals

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

### Resource not refetching

If the input signal to the resource is a collection that changes internally (like the `Set` example above), then SolidJS will not notice the change and thus not refetch the resource. Creating a new set each time seems to be the best approach.

### Fetching resource when URI changes

There appears to be an issue with `useParams()` and `createResource()` such that the resource is not refreshed when the identifier in the path parameter changes. However, `createEffect()` will detect the change and as such can be used, along with `refetch` to force the resource to reload. See also https://github.com/solidjs/solid/discussions/1745

```javascript
  const params = useParams();
  const client = useApolloClient();
  const [assetQuery, { refetch }] = createResource(
    () => params.id,
    async (assetId) => {
      const { data } = await client.query({
        query: GET_ASSET,
        variables: { id: assetId }
      });
      return data;
    }
  );
  const location = useLocation();
  createEffect(() => refetch(location.pathname));
```

### Form fields not updating when resource loads

If using `Suspense` then may need to switch to `Show` and add the `keyed` attribute. Without this, SolidJS seemingly does not notice that the resource changed and thus all of the elements related to that resource need to be rebuilt.
