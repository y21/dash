const ZERO = '0'.charCodeAt();
const NINE = '9'.charCodeAt();
const ALC_CODE = 'a'.charCodeAt();
const ZLC_CODE = 'z'.charCodeAt();
const AUC_CODE = 'A'.charCodeAt();
const ZUC_CODE = 'Z'.charCodeAt();

const KEYWORDS = new Set([
    'new'
]);

class Lexer {
    tokens = [];
    index = 0;
    constructor(source) {
        this.source = source;
    }

    lexAll() {
        while (this.index < this.source.length) {
            const cur = this.source[this.index];
            switch (cur) {
                // whitespace, just skip
                case " ":
                case "\n":
                    this.index++;
                    break;
                // operators like ++ -- **
                case "+":
                case "-":
                case "*":
                case "=":
                    if (this.source[this.index + 1] === cur) {
                        this.tokens.push({ type: 'ExprOperator', value: cur + cur });
                        this.index += 2;
                        break;
                    }
                case "?":
                case ":":
                case "/":
                case "(":
                case ")":
                case "%":
                case ",":
                case ">":
                case "<":
                    this.index++;
                    this.tokens.push({ type: 'ExprOperator', value: cur });
                    break;
                default:
                    // numbers
                    const code = cur.charCodeAt(0);
                    if (code >= ZERO && code <= NINE) {
                        this.lexNumber();
                    } else if ((code >= ALC_CODE && code <= ZLC_CODE) || (code >= AUC_CODE && code <= ZUC_CODE)) {
                        this.lexIdentifier();
                    } else {
                        throw new Error('unknown character: ' + cur);
                    }
            }
        }

        return this.tokens;
    }

    lexNumber() {
        let num = "";
        while (this.index < this.source.length) {
            const c = this.source.charCodeAt(this.index);
            if (c >= ZERO && c <= NINE) {
                num = num + String.fromCharCode(c);
            } else {
                break;
            }
            this.index++;
        }
        this.tokens.push({ type: 'Literal', value: parseInt(num) });
    }

    lexIdentifier() {
        let num = "";
        while (this.index < this.source.length) {
            const c = this.source.charCodeAt(this.index);
            if ((c >= ALC_CODE && c <= ZLC_CODE) || (c >= AUC_CODE && c <= ZUC_CODE)) {
                num += String.fromCharCode(c);
            } else {
                break;
            }
            this.index++;
        }
        if (KEYWORDS.has(num)) {
            this.tokens.push({ type: 'Keyword', value: num });
        } else {
            this.tokens.push({ type: 'Identifier', value: num });
        }
    }
}



/**
 * Precedence Table for this parser (highest to lowest):
 * 
 * - Literal expressions and grouping: 1, abc, "def", (1+2)
 * - Conditional: a ? b : c
 * - Assignment: a = b
 * - Prefix increment/decrement: ++foo, --foo
 * - Exponentiation: a ** b
 * - Factor: a * b, a / b
 * - Term: a + b, a - b
 */
class Parser {
    index = 0;
    constructor(source) {
        this.source = source;
    }

    // HELPER METHODS

    current() {
        return this.source[this.index];
    }

    previous() {
        return this.source[this.index - 1];
    }

    match(...values) {
        const cur = this.current();
        if (cur && cur.type === 'ExprOperator' && values.includes(cur.value)) {
            this.index++;
            return true;
        }
        return false;
    }

    matchKeyword(keyword) {
        const cur = this.current();
        if (cur && cur.type === 'Keyword' && keyword === cur.value) {
            this.index++;
            return true;
        }
        return false;
    }

    parseExpression() {
        return this.term();
    }

    term() {
        let left = this.factor();

        while (this.match('+', '-')) {
            const operator = this.previous();
            const right = this.factor();
            left = {
                type: 'BinaryExpr',
                operator: operator,
                left: left,
                right: right
            };
        }

        return left;
    }

    factor() {
        let left = this.exponentiation();

        while (this.match('*', '/', '%')) {
            const operator = this.previous();
            const right = this.exponentiation();
            left = {
                type: 'BinaryExpr',
                operator: operator,
                left: left,
                right: right
            };
        }

        return left;
    }

    exponentiation() {
        let left = this.prefix();

        if (this.match('**')) {
            const operator = this.previous();
            // ** is right associative, so it's self recursive
            // instead of a while (this.match('**')) loop
            const right = this.exponentiation();
            left = {
                type: 'BinaryExpr',
                operator: operator,
                left: left,
                right: right
            };
        }

        return left;
    }

    prefix() {
        // ++foo and --foo and -foo and +foo is unary, so it immediately begins with the operator
        if (this.match('++', '--', '+', '-')) {
            const operator = this.previous();
            const right = this.prefix();
            return {
                type: 'UnaryExpr',
                operator: operator,
                right: right
            };
        }

        return this.assignment();
    }

    assignment() {
        let left = this.conditional();

        if (this.match('=')) {
            const right = this.assignment();
            left = {
                type: 'AssignmentExpr',
                left: left,
                right: right
            };
        }

        return left;
    }

    conditional() {
        let condition = this.equality();

        if (this.match('?')) {
            let thenBranch = this.parseExpression();
            if (!this.match(':')) {
                throw new Error('Expected ":" in conditional expr');
            }
            const elseBranch = this.parseExpression();
            condition = {
                type: 'ConditionalExpr',
                condition: condition,
                thenBranch: thenBranch,
                elseBranch: elseBranch
            };
        }

        return condition;
    }

    equality() {
        let left = this.comparison();

        while (this.match('==')) {
            const operator = this.previous();
            const right = this.comparison();
            left = { type: 'BinaryExpr', operator: operator, left: left, right: right };
        }

        return left;
    }

    comparison() {
        let left = this.call();

        while (this.match('<', '>')) {
            const operator = this.previous();
            const right = this.call();
            left = { type: 'BinaryExpr', operator: operator, left: left, right: right };
        }

        return left;
    }

    // Function calls
    call() {
        let target = this.literal();

        while (this.match('(')) {
            const args = [];

            while (!this.match(')')) {
                if (args.length > 0) {
                    this.match(',');
                }

                args.push(this.parseExpression());
            }

            target = {
                type: 'FunctionCall',
                target: target,
                args: args
            };
        }

        return target;
    }

    literal() {
        const cur = this.current();
        this.index++;

        switch (cur.type) {
            case 'Literal':
                return { type: 'LiteralExpr', value: cur.value };
            case 'Identifier':
                return { type: 'IdentifierExpr', value: cur.value };
            case 'ExprOperator':
                switch (cur.value) {
                    case '(':
                        const values = [];
                        while (!this.match(')')) {
                            if (values.length > 0) {
                                this.match(',');
                            }

                            values.push(this.parseExpression());
                        }

                        // We only know if it's a lambda function after parsing all the expressions in the ()
                        let isLambda = this.match('-') && this.match('>');

                        if (isLambda) {
                            const parameters = [];
                            // for (const { type, value } of values) {
                            for (const entry of values) {
                                const type = entry.type;
                                const value = entry.value;
                                if (type !== 'IdentifierExpr') {
                                    throw new Error('parameter must be an identifier');
                                }
                                parameters.push(value);
                            }
                            const body = this.parseExpression();

                            return { type: 'LambdaExpr', parameters: parameters, body: body };
                        } else {
                            // Otherwise it's a grouping expression, e.g. (2 * 3)

                            if (values.length !== 1) {
                                throw new Error('Grouping expression must contain exactly one element');
                            }

                            return { type: 'GroupingExpr', value: values[0] };
                        }
                }
        }

        throw new Error(`Unexpected token: ${cur.value}`);
    }
}

class Evaluator {
    function = null;
    variables = {};

    evaluateNode(node) {
        switch (node.type) {
            case 'BinaryExpr':
                switch (node.operator.value) {
                    case '-': return this.evaluateNode(node.left) - this.evaluateNode(node.right);
                    case '+': return this.evaluateNode(node.left) + this.evaluateNode(node.right);
                    case '*': return this.evaluateNode(node.left) * this.evaluateNode(node.right);
                    case '/': return this.evaluateNode(node.left) / this.evaluateNode(node.right);
                    case '**': return this.evaluateNode(node.left) ** this.evaluateNode(node.right);
                    case '%': return this.evaluateNode(node.left) % this.evaluateNode(node.right);
                    case '==': return this.evaluateNode(node.left) == this.evaluateNode(node.right);
                    case '>': return this.evaluateNode(node.left) > this.evaluateNode(node.right);
                    case '<': return this.evaluateNode(node.left) < this.evaluateNode(node.right);
                    default: throw new Error('unhandled binary expression operator');
                }
            case 'IdentifierExpr': {
                if (node.value === 'self') return this.function;
                else return this.variables[node.value];
            }
            case 'LiteralExpr':
                return node.value;
            case 'GroupingExpr': return this.evaluateNode(node.value);
            case 'ConditionalExpr': {
                const condition = this.evaluateNode(node.condition);
                if (condition) return this.evaluateNode(node.thenBranch);
                else return this.evaluateNode(node.elseBranch);
            }
            case 'UnaryExpr': {
                // Check if this expression is --foo or ++foo
                if (['--', '++'].includes(node.operator.value)) {

                    // The target type must be an identifier, because for example ++6 is not allowed.
                    if (node.right.type !== 'IdentifierExpr') {
                        throw new Error('invalid place to assign to, must be an identifier');
                    }

                    const key = node.right.value;
                    let prev = this.variables[key];
                    if (prev === undefined) {
                        throw new Error(`${key} is not defined`);
                    }

                    // Simply increment or decrement the variable
                    switch (node.operator.value) {
                        case '--': {
                            this.variables[key] = this.variables[key] - 1;
                            break;
                        }
                        case '++': {
                            this.variables[key] = this.variables[key] + 1;
                            break;
                        }
                    }

                    return prev;
                } else {
                    // If it isn't -- or ++, then it is either -foo or -bar
                    switch (node.operator.value) {
                        case '-': return -(this.evaluateNode(node.right));
                        case '+': return +(this.evaluateNode(node.right));
                    }
                }
            }
            case 'FunctionCall':
                const target = this.evaluateNode(node.target);
                const args = [];
                for (const arg of node.args) {
                    args.push(this.evaluateNode(arg));
                }

                // We have two types of functions: native JavaScript functions, which are written directly in JavaScript
                // And we have lambdas
                if (typeof target === 'function') {
                    // Native functions are easy, just call them directly
                    // TODO: implement spread in function call
                    return target(args);
                    // return target(...args);
                } else if (typeof target === 'object' && target.type === 'LambdaExpr') {
                    // For lambda functions, we create a new evaluator with its own variables
                    // TODO: Evaluator should probably have a reference to the parent variables,
                    // to make it possible to access variables from outer scope, maybe??
                    const subevaluator = new Evaluator();
                    subevaluator.function = target;

                    for (let i = 0; i < args.length; i++) {
                        const paramName = target.parameters[i];
                        const paramValue = args[i];
                        subevaluator.variables[paramName] = paramValue;
                    }

                    return subevaluator.evaluateNode(target.body);
                } else {
                    throw new Error('value is not a function');
                }
            case 'LambdaExpr':
                return node;
            default: throw new Error('unhandled node ' + node.type);
        }
    }
}

function interpretCode(code, variables) {
    if (variables === undefined) {
        variables = {};
    }
    const tokens = new Lexer(code).lexAll();
    const ast = new Parser(tokens).parseExpression();
    const evaluator = new Evaluator();

    // Object.setPrototypeOf(variables, null);
    evaluator.variables = variables;

    return evaluator.evaluateNode(ast);
}

const result = interpretCode(` 
    ((n) -> 
        (n == 0)
            ? n
            : n + self(n-1)
    )(50)
`);

result
