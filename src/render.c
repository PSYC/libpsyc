#include <stdio.h>

#include "lib.h"
#include <psyc/packet.h>
#include <psyc/render.h>
#include <psyc/syntax.h>

#ifdef __INLINE_PSYC_RENDER
static inline
#endif
PsycRenderRC
psyc_render_list (PsycList *list, char *buffer, size_t buflen)
{
    size_t i, cur = 0;
    PsycString *elem;

    if (list->length > buflen) // return error if list doesn't fit in buffer
	return PSYC_RENDER_ERROR;

    if (list->flag == PSYC_LIST_NEED_LENGTH) {
	for (i = 0; i < list->num_elems; i++) {
	    elem = &list->elems[i];
	    if (i > 0)
		buffer[cur++] = '|';
	    cur += itoa(elem->length, buffer + cur, 10);
	    buffer[cur++] = ' ';
	    memcpy(buffer + cur, elem->data, elem->length);
	    cur += elem->length;
	}
    } else {
	for (i = 0; i < list->num_elems; i++) {
	    elem = &list->elems[i];
	    buffer[cur++] = '|';
	    memcpy(buffer + cur, elem->data, elem->length);
	    cur += elem->length;
	}
    }

#ifdef DEBUG
    // Actual length should be equal to pre-calculated length at this point.
    assert(cur == list->length);
#endif
    return PSYC_RENDER_SUCCESS;
}

PsycRenderRC
psyc_render_table (PsycTable *table, char *buffer, size_t buflen)
{
    size_t cur = 0;

    if (table->length > buflen) // return error if table doesn't fit in buffer
	return PSYC_RENDER_ERROR;

    if (table->width > 0) {
	cur = sprintf(buffer, "*%ld", table->width);
	buffer[cur++] = ' ';
    }

    return psyc_render_list(table->list, buffer + cur, buflen - cur);
}

static inline size_t
psyc_render_modifier (PsycModifier *mod, char *buffer)
{
    size_t cur = 0;

    buffer[cur++] = mod->oper;
    memcpy(buffer + cur, mod->name.data, mod->name.length);
    cur += mod->name.length;
    if (cur == 1)
	return cur; // error, name can't be empty

    if (mod->flag == PSYC_MODIFIER_NEED_LENGTH) {
	buffer[cur++] = ' ';
	cur += itoa(mod->value.length, buffer + cur, 10);
    }

    buffer[cur++] = '\t';
    memcpy(buffer + cur, mod->value.data, mod->value.length);
    cur += mod->value.length;
    buffer[cur++] = '\n';

    return cur;
}

#ifdef __INLINE_PSYC_RENDER
static inline
#endif
PsycRenderRC
psyc_render (PsycPacket *packet, char *buffer, size_t buflen)
{
    size_t i, cur = 0, len;

    if (packet->length > buflen) // return error if packet doesn't fit in buffer
	return PSYC_RENDER_ERROR;

    // render routing modifiers
    for (i = 0; i < packet->routing.lines; i++) {
	len = psyc_render_modifier(&packet->routing.modifiers[i], buffer + cur);
	cur += len;
	if (len <= 1)
	    return PSYC_RENDER_ERROR_MODIFIER_NAME_MISSING;
    }

    // add length if needed
    if (packet->flag == PSYC_PACKET_NEED_LENGTH)
	cur += itoa(packet->contentlen, buffer + cur, 10);

    if (packet->flag == PSYC_PACKET_NEED_LENGTH || packet->content.length
	|| packet->stateop || packet->entity.lines
	|| packet->method.length || packet->data.length)
	buffer[cur++] = '\n'; // start of content part if there's content or length

    if (packet->content.length) { // render raw content if present
	memcpy(buffer + cur, packet->content.data, packet->content.length);
	cur += packet->content.length;
    } else {
	if (packet->stateop) {
	    buffer[cur++] = packet->stateop;
	    buffer[cur++] = '\n';
	}
	// render entity modifiers
	for (i = 0; i < packet->entity.lines; i++)
	    cur += psyc_render_modifier(&packet->entity.modifiers[i],
					buffer + cur);

	if (packet->method.length) { // add method\n
	    memcpy(buffer + cur, packet->method.data, packet->method.length);
	    cur += packet->method.length;
	    buffer[cur++] = '\n';

	    if (packet->data.length) { // add data\n
		memcpy(buffer + cur, packet->data.data, packet->data.length);
		cur += packet->data.length;
		buffer[cur++] = '\n';
	    }
	} else if (packet->data.length)	// error, we have data but no modifier
	    return PSYC_RENDER_ERROR_METHOD_MISSING;
    }

    // add packet delimiter
    buffer[cur++] = PSYC_PACKET_DELIMITER_CHAR;
    buffer[cur++] = '\n';

    // actual length should be equal to pre-calculated length at this point
    assert(cur == packet->length);
    return PSYC_RENDER_SUCCESS;
}

PsycRenderRC
psyc_render_packet_id (char *context, size_t contextlen,
		       char *source, size_t sourcelen,
		       char *target, size_t targetlen,
		       char *counter, size_t counterlen,
		       char *fragment, size_t fragmentlen,
		       char *buffer, size_t buflen)
{
    PsycList list;
    PsycString elems[PSYC_PACKET_ID_ELEMS] = {};

    if (contextlen)
	elems[PSYC_PACKET_ID_CONTEXT] = PSYC_STRING(context, contextlen);
    if (sourcelen)
	elems[PSYC_PACKET_ID_SOURCE] = PSYC_STRING(source, sourcelen);
    if (targetlen)
	elems[PSYC_PACKET_ID_TARGET] = PSYC_STRING(target, targetlen);
    if (counterlen)
	elems[PSYC_PACKET_ID_COUNTER] = PSYC_STRING(counter, counterlen);
    if (fragmentlen)
	elems[PSYC_PACKET_ID_FRAGMENT] = PSYC_STRING(fragment, fragmentlen);

    psyc_list_init(&list, elems, PSYC_PACKET_ID_ELEMS, PSYC_LIST_NO_LENGTH);
    return psyc_render_list(&list, buffer, buflen);
}
