#ifndef SIMPLICITY_CTX8UNPRUNED_H
#define SIMPLICITY_CTX8UNPRUNED_H

#include <stddef.h>
#include <stdint.h>

/* A length-prefixed encoding of the following Simplicity program:
 *     (scribe (toWord256 0x067C531269735CA7F541FDACA8F0DC76305D3CADA140F89372A410FE5EFF6E4D) &&&
 *      (ctx8Init &&& scribe (toWord128 0xDE188941A3375D3A8A061E67576E926D)) >>> ctx8Addn vector16 >>> ctx8Finalize) >>>
 *     eq >>> verify
 */
extern const unsigned char ctx8Unpruned[];
extern const size_t sizeof_ctx8Unpruned;

/* The commitment Merkle root of the above ctx8Unpruned Simplicity expression. */
extern const uint32_t ctx8Unpruned_cmr[];

/* The identity Merkle root of the above ctx8Unpruned Simplicity expression. */
extern const uint32_t ctx8Unpruned_imr[];

/* The annotated Merkle root of the above ctx8Unpruned Simplicity expression. */
extern const uint32_t ctx8Unpruned_amr[];

#endif
