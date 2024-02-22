import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";
import "./common.tsp";
import "./user.tsp";
import "./rent.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;

@tag("book")
@route("/books")
namespace KMNLib.BookAPI {
    @format("uuid")
    scalar BookId extends string;
    model CreateBook {
        title: string;
        amount: int32;
    }

    model UpdateBook {
        title?: string;
        amount?: int32;
    }

    model BookResponse {
        id: BookId;
        title: string;
        amount: int32;
    }

    model BookCreatedReponse {
        id: BookId;
    }

    interface Books {
        @summary("Get All Books")
        @get
        get(): {
            ...Common.Success;
            @body book: [BookResponse];
        } | Common.InternalError;
        @summary("Create book")
        @post
        post(
            @body
            body: CreateBook,
        ): {
            ...Common.Success;

            @body
            body: BookCreatedReponse;
        } | Common.InternalError;
    }
    @route("/{id}")
    namespace Book {
        @summary("Get book from id")
        @get
        op get(
            @path
            id: BookId,
        ): {
            ...Common.Success;

            @body
            body: BookResponse;
        } | Common.InternalError | Common.NotFound;
        @summary("Update book data")
        @patch
        op patch(
            @path
            id: BookId,

            @body
            body: UpdateBook,
        ): Common.Success | Common.InternalError;
        @summary("Delete book")
        @delete
        op delete(
            @path
            id: BookId,
        ): {
            ...Common.Success;
            @body body: BookId;
        } | Common.InternalError;

        @route("/rents")
        interface Rents {
            @summary("Get user rent book informations")
            @get
            get(
                @path
                id: BookId,
            ): {
                ...Common.Success;

                @body
                body: RentAPI.RentResponse;
            } | Common.InternalError | Common.NotFound;
        }
    }
}
