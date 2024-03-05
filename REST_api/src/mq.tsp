import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";
import "./common.tsp";
import "./main.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;

@tag("queue")
@route("/queue")
namespace KMNLib.QueueStateAPI;

@format("uuid")
scalar InfoId extends string;

alias InfoTarget = "delayed" | "failed";

model GetInfos {
  target: InfoTarget;
  size?: int64 = 0;
  offset?: int64 = 0;
}

model InfoResponse {
  id: InfoId;
  data: string;
  stack_trace: string;
}

model InfoLengthResponse {
  length: uint64;
}

@route("/infos")
namespace Infos {
  @get
  op get(
    @body
    body: GetInfos,
  ): {
    ...Common.Success;
    @body data: InfoResponse[];
  } | Common.InternalError;

  @route("/{id}")
  interface Info {
    @get
    get(
      @path
      id: InfoId,
    ): {
      ...Common.Success;

      @body
      body: InfoResponse;
    } | Common.InternalError | Common.NotFound;
  }

  @route("/len")
  interface Length {
    @get
    get(
      @body
      body: {
        target: InfoTarget | "queued";
      },
    ): {
      ...Common.Success;

      @body
      body: InfoLengthResponse;
    } | Common.InternalError;
  }
}
