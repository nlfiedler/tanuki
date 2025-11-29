//
// Copyright (c) 2025 Nathan Fiedler
//
import { useParams } from '@solidjs/router'

function AssetDetails() {
  const params = useParams()
  return (
    <div>
      <p>Details for asset {params.id}.</p>
    </div>
  )
}

export default AssetDetails
