import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";
import "./common.tsp";
import "./user.tsp";
import "./book.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;

@tag("rent")
@route("/rents")
namespace KMNLib.RentAPI {
  model Rent {
    user_id: UserAPI.UserId;
    book_id: BookAPI.BookId;
  }
  model Return {
    user_id: UserAPI.UserId;
    book_id: BookAPI.BookId;
  }
  model RentResponse {
    user_id: UserAPI.UserId;
    book_id: BookAPI.BookId;
    returned_at?: utcDateTime;
  }

  interface Rents {
    @post
    post(
      @body
      body: Rent
    ): Common.Success | Common.InternalError | Common.Conflict;
    @delete
    delete(
      @body
      body: Rent
    ): Common.Success | Common.InternalError | Common.Conflict;
  }
}
