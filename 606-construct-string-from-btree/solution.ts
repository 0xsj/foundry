class TreeNode {
  val: number;
  left: TreeNode | null;
  right: TreeNode | null;
  constructor(val?: number, left?: TreeNode | null, right?: TreeNode | null) {
    this.val = val === undefined ? 0 : val;
    this.left = left === undefined ? null : left;
    this.right = right === undefined ? null : right;
  }
}

function treeToString(root: TreeNode | null): string {
  if (!root) {
    return "";
  }
  const result: string[] = [];
  preorderTraversal(root, result);
  return result.join("");
}

function preorderTraversal(node: TreeNode | null, result: string[]): void {
  if (!node) {
    return;
  }

  result.push(node.val.toString());

  if (node.left || node.right) {
    result.push("(");
    preorderTraversal(node.left, result);
    result.push(")");
  }

  if (node.right) {
    result.push("(");
    preorderTraversal(node.right, result);
    result.push(")");
  }
}
const root = new TreeNode(1, new TreeNode(2, new TreeNode(4)), new TreeNode(3));
const result = treeToString(root);
console.log(result);
