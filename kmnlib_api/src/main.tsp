import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";
import "./book.tsp";
import "./common.tsp";
import "./mq.tsp";
import "./rent.tsp";
import "./user.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;

/** Note: Auth system is not planned */
@service({
  title: "KMNLib REST API",
  version: "1.0.0",
})
@server("/api/v1", "The service endpoint")
namespace KMNLib;
