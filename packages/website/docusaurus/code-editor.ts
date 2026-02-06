import { visit } from 'unist-util-visit';
import type { Root, Node, Code } from 'mdast';

/**
 * A remark plugin to transform code blocks with language 'mira' or 'miratpl'
 * into custom 'miraCodeBlock' nodes for further processing.
 */
export default function remarkCodeEditor() {
    return (tree: Root): void => {
        visit(tree, { type: 'code' }, (node: Node) => {
            const codeNode = node as Code;
            if (codeNode.lang !== 'mira' && codeNode.lang !== 'miratpl') {
                return;
            }
            node.type = 'miraCodeBlock';
            node.data = {
                hName: 'mira',
                hProperties: {
                    title: codeNode.meta || '',
                    value: codeNode.value,
                    mode: codeNode.lang === 'miratpl' ? 'Template' : 'Script',
                },
            };
        });
    };
}
