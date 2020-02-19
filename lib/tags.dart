//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:graphql_flutter/graphql_flutter.dart';

// Name the query so the mutations can invoke in refetchQueries.
const String queryTags = """
  query getAllTags {
    tags {
      value
      count
    }
  }
""";

/// Display the available asset tags.
class TagsList extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Query(
      options: QueryOptions(
        documentNode: gql(queryTags),
      ),
      builder: (QueryResult result,
          {VoidCallback refetch, FetchMore fetchMore}) {
        if (result.hasException) {
          return Text(result.exception.toString());
        }
        if (result.loading) {
          return Text('Loading');
        }
        List tags = result.data['tags'];
        return ListView.builder(
          itemCount: tags.length,
          itemBuilder: (context, index) {
            final tag = tags[index];
            return Text(tag['value'] + ': ' + tag['count'].toString());
          },
        );
      },
    );
  }
}
